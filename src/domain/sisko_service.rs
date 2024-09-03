use super::{
    Album, AlbumRepository, AudioFile, AudioFileRepository, DomainEvent, File, FileService,
    LogHistory,
};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::musicbrainz::{MusicBrainzService, Release, ReleaseLookup};
use crate::ui::UiWrapper;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File as FsFile;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
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
        SiskoService {}
    }

    pub fn add_file(&self, file: Arc<File>) -> Result<()> {
        let mut audio_file = AudioFile::from(file.as_ref());
        audio_file
            .events
            .push(DomainEvent::AudioFileAdded(audio_file.clone()));
        AudioFileRepository::instance().add(audio_file.clone())?;
        Ok(())
    }

    pub fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = FileService::instance().get_files_in_dir_recursive(&file.absolute_path)?;
        for file in files {
            self.add_file(file)?;
        }
        Ok(())
    }

    pub fn calculate_fingerprint(&self, audio_file: &AudioFile) -> Result<()> {
        let path = audio_file.file.absolute_path.clone();
        let fingerprint = AcoustIdService::instance().get_fingerprint(&path).ok();
        let mut audio_file = audio_file.clone();
        audio_file.fingerprint = fingerprint.or(audio_file.fingerprint.clone());
        audio_file
            .events
            .push(DomainEvent::AudioFileUpdated(audio_file.clone()));
        AudioFileRepository::instance().save(audio_file)?;
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
        UiWrapper::instance().open_logs(&logs)?;
        Ok(())
    }

    pub fn save_audio_file(&self, audio_file: &AudioFile) -> Result<()> {
        let audio_bytes = Self::get_audio_bytes(&audio_file.file.absolute_path)?;
        let filename = &audio_file.file.name;
        let tag = audio_file.tags.first().ok_or_else(|| anyhow!(""))?;
        let tag = ID3v2Tag::from(tag);
        let mut bytes = tag.to_bytes();
        bytes.extend(audio_bytes);
        let mut file = FsFile::create(filename)?;
        file.write_all(&bytes)?;
        // if moved, replace original w/ copy w/ new file
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

    pub fn select_audio_file(&self, audio_file: &AudioFile) -> Result<()> {
        UiWrapper::instance().set_metadata_table(audio_file)
    }

    pub async fn scan_audio_file(&self, audio_file: &AudioFile) -> Result<()> {
        // fingerprint
        let file_path = audio_file.file.absolute_path.clone();
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
        let albums = self.load_lookup(&lookup)?;
        let mut album = self.choose_album(&albums)?;
        album.match_file(&audio_file)?;
        album.update_tag_fields(&mut audio_file)?;
        audio_file
            .events
            .push(DomainEvent::AudioFileUpdated(audio_file.clone()));
        AudioFileRepository::instance().save(audio_file.clone())?;

        // TODO: replace with event handler for AudioFileMatched
        UiWrapper::instance().remove_cluster_file(&audio_file)?;
        UiWrapper::instance().add_album(&album)?;
        Ok(())
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

    fn load_lookup(&self, lookup: &ReleaseLookup) -> Result<Vec<Album>> {
        lookup
            .releases
            .iter()
            .map(|r| self.load_release(r))
            .collect()
    }

    fn load_release(&self, release: &Release) -> Result<Album> {
        let album = Album::from(release);
        AlbumRepository::instance().add(album.clone());
        Ok(album)
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
