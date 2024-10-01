use crate::domain::events::DomainEvent;
use crate::domain::models::{Album, AudioFile, AudioFileId, Metadata, Track};
use crate::domain::repos::{AlbumRepository, AudioFileRepository, TagRepository, TrackRepository};
use crate::domain::services::LogHistory;
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::file::{File, FileService};
use crate::infrastructure::musicbrainz::{MusicBrainzService, Release, ReleaseLookup};
use crate::infrastructure::spawn;
use crate::ui::models::MatchState;
use crate::ui::services::Ui;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File as FsFile;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Represents a service for application actions.
pub struct SiskoService {}

impl SiskoService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<SiskoService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new service for application actions.
    pub fn new() -> Self {
        Self {}
    }

    pub async fn add_file(&self, file: Arc<File>) -> Result<()> {
        let mut audio_file = AudioFile::from(file.as_ref());
        AudioFileRepository::instance()
            .add(audio_file.clone())
            .await?;
        self.load_tags(file.as_ref()).await?;
        audio_file
            .events
            .push(DomainEvent::AudioFileAdded(audio_file.clone()));
        AudioFileRepository::instance().save(audio_file).await
    }

    pub async fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = FileService::instance().get_files_in_dir_recursive(&file.absolute_path)?;
        for file in files {
            self.add_file(file).await?;
        }
        Ok(())
    }

    pub async fn calculate_fingerprint(&self, audio_file: &AudioFile) -> Result<()> {
        let path = audio_file.id.path.clone();
        let fingerprint = AcoustIdService::instance().get_fingerprint(&path).ok();
        let mut audio_file = audio_file.clone();
        audio_file.fingerprint = fingerprint.or(audio_file.fingerprint.clone());
        audio_file
            .events
            .push(DomainEvent::AudioFileUpdated(audio_file.clone()));
        AudioFileRepository::instance().save(audio_file).await
    }

    pub fn handle_audio_file_added(&self, audio_file: &AudioFile) {
        let copy = audio_file.clone();
        spawn(async move { SiskoService::instance().calculate_fingerprint(&copy).await });
        let copy = audio_file.clone();
        spawn(async move {
            let tag = TagRepository::instance().get(&copy).await?;
            Ui::instance().cluster_table.add_cluster_file(copy, &tag)
        });
    }

    pub fn handle_audio_file_updated(&self, audio_file: &AudioFile) {
        let audio_file = audio_file.clone();
        spawn(async move {
            let tag = TagRepository::instance().get(&audio_file).await?;
            Ui::instance()
                .cluster_table
                .add_cluster_file(audio_file, &tag)
        });
    }

    pub fn handle_tag_updated(&self, metadata: &Metadata) {
        let metadata = metadata.clone();
        spawn(async move {
            let ui = Ui::instance();
            let audio_file = AudioFileRepository::instance()
                .get(&metadata.audio_file_id)
                .await?;
            let metadata = TagRepository::instance().get(&audio_file).await?;
            // TODO: this doesn't seem right
            ui.cluster_table
                .add_cluster_file(audio_file.clone(), &metadata)?;
            ui.album_table.update_audio_file(&audio_file, &metadata)?;
            ui.metadata_table.update_metadata_table(&metadata)
        });
    }

    pub async fn load_tags(&self, file: &File) -> Result<()> {
        // TODO: check if has tag before reading
        if let Ok(id3v2) = ID3v2Tag::read_from_path(&file.absolute_path) {
            let metadata =
                Metadata::from_id3v2(AudioFileId::new(file.absolute_path.clone()), &id3v2.frames);
            TagRepository::instance().add(metadata).await?;
        }

        // TODO: add loading other possible tags
        Ok(())
    }

    pub fn open_logs(&self) -> Result<()> {
        let logs = LogHistory::instance()
            .logs()
            .lock()
            .map_err(|_| anyhow!("Error unlocking the logs mutex!"))?
            .join("")
            .lines()
            .rev()
            .join("");
        Ui::instance().menu.open_logs(&logs)?;
        Ok(())
    }

    pub async fn remove_file(&self, audio_file: &AudioFile) -> Result<()> {
        let mut audio_file = audio_file.clone();
        audio_file
            .events
            .push(DomainEvent::AudioFileRemoved(audio_file.id.clone()));
        AudioFileRepository::instance().remove(audio_file).await
    }

    pub async fn save_audio_file(&self, audio_file: &AudioFile) -> Result<()> {
        let audio_bytes = Self::get_audio_bytes(&audio_file.id.path)?;
        let filename = &audio_file
            .id
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let tag = TagRepository::instance().get(audio_file).await?;
        let tag = ID3v2Tag::from(&tag);
        let mut bytes = tag.to_bytes();
        bytes.extend(audio_bytes);
        let mut file = FsFile::create(filename)?;
        file.write_all(&bytes)?;

        // TODO: if moved, replace original w/ copy w/ new file
        let new_file = Path::new(filename);
        let new_file = FileService::instance().get(new_file)?;
        if new_file.absolute_path != audio_file.id.path {
            // TODO: copy data from old to new
            self.add_file(new_file).await?;
            self.remove_file(audio_file).await?;
        }

        Ok(())
    }

    fn get_audio_bytes(path: &PathBuf) -> Result<Vec<u8>> {
        let offset = match ID3v2Tag::read_from_path(path).ok() {
            Some(tag) => tag.total_size(),
            None => 0,
        };

        let mut file = FsFile::open(path)?;
        file.seek(SeekFrom::Start(offset.into()))?;
        let mut file_content: Vec<u8> = vec![];
        file.read_to_end(&mut file_content)?;
        Ok(file_content)
    }

    pub async fn select_audio_file(&self, audio_file_id: &AudioFileId) -> Result<()> {
        let audio_file = AudioFileRepository::instance().get(audio_file_id).await?;
        let tag = TagRepository::instance().get(&audio_file).await?;
        Ui::instance().metadata_table.set_metadata_table(&tag)
    }

    pub async fn scan_audio_file(&self, audio_file: &AudioFile) -> Result<()> {
        // fingerprint
        let file_path = audio_file.id.path.clone();
        let fingerprint = AcoustIdService::instance().get_fingerprint(&file_path)?;

        // acoustid
        let lookup = AcoustIdService::instance()
            .lookup_fingerprint(&fingerprint)
            .await?;

        // recording
        let recordingid = lookup[0].recordings[0].id.clone();
        let mut audio_file = audio_file.clone();
        audio_file.acoust_id = Some(lookup[0].id.clone());
        audio_file.recording_id = Some(recordingid.clone());

        // lookup
        let lookup = MusicBrainzService::instance()
            .lookup_releases_for_recording(&recordingid)
            .await?;

        // get album
        let albums = self.load_lookup(&lookup).await?;
        let album = self.choose_album(&albums)?;
        let tracks = TrackRepository::instance().get_all(&album).await?;
        let matched_track = audio_file.match_to_album(&album, &tracks)?;
        let mut updated_tag = TagRepository::instance().get(&audio_file).await?;
        updated_tag.update_for_match(&audio_file, &album, &matched_track);
        audio_file
            .events
            .push(DomainEvent::AudioFileUpdated(audio_file.clone()));
        AudioFileRepository::instance()
            .save(audio_file.clone())
            .await?;
        TagRepository::instance().save(updated_tag).await?;
        let match_states = MatchState::for_tracks(&tracks).await?;

        // TODO: replace with event handler for AudioFileMatched
        Ui::instance()
            .cluster_table
            .remove_cluster_file(&audio_file)?;
        Ui::instance()
            .album_table
            .add_album(&album, &tracks, &match_states)?;
        Ok(())
    }

    pub fn update_ui(&self) {
        spawn(async move {
            let audio_files = AudioFileRepository::instance().get_all().await?;
            for audio_file in audio_files.into_iter().filter(|f| f.track_id.is_none()) {
                let metadata = TagRepository::instance().get(&audio_file).await?;
                Ui::instance()
                    .cluster_table
                    .add_cluster_file(audio_file, &metadata)?;
            }

            let albums = AlbumRepository::instance().get_all().await?;
            for album in &albums {
                let tracks = TrackRepository::instance().get_all(album).await?;
                let match_states = MatchState::for_tracks(&tracks).await?;
                if match_states.iter().any(|s| s.is_matched()) {
                    Ui::instance()
                        .album_table
                        .add_album(album, &tracks, &match_states)?;
                }
            }
            Ok(())
        });
    }

    fn choose_album(&self, albums: &[Album]) -> Result<Album> {
        /*let (release, _) = lookup
            .releases
            .iter()
            .map(|r| (r, self.get_priority(&r)))
            .max_by(|(_, p1), (_, p2)| p1.total_cmp(p2))
            .unwrap();
        Ok(release.id.clone())*/
        Ok(albums[0].clone())
    }

    fn get_priority(&self, release: &Release) -> f64 {
        0.5
    }

    async fn load_lookup(&self, lookup: &ReleaseLookup) -> Result<Vec<Album>> {
        let mut albums = vec![];
        for release in &lookup.releases {
            let album = self.load_release(release).await?;
            albums.push(album);
        }
        Ok(albums)
    }

    async fn load_release(&self, release: &Release) -> Result<Album> {
        let album = Album::from(release);
        AlbumRepository::instance().add(album.clone()).await?;

        let tracks: Vec<Track> = release
            .media
            .iter()
            .flat_map(|media| {
                media
                    .tracks
                    .iter()
                    .map(|track| Track::new(&release.id, media, track))
            })
            .collect();
        for chunk in tracks.chunks(10) {
            TrackRepository::instance().add_all(chunk.to_vec()).await?;
        }
        //
        Ok(album)
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
