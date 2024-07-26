use super::{Album, AudioFile};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::musicbrainz::{MusicBrainzService, Release, ReleaseLookup};
use crate::infrastructure::{Am, Ram};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct AlbumService {
    albums: Mutex<HashMap<String, Am<Album>>>,
}

impl AlbumService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AlbumService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            albums: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_album_for_file(&self, audio_file: &Am<AudioFile>) -> Ram<Album> {
        // fingerprint
        let file_path = {
            let audio_file = audio_file
                .lock()
                .map_err(|_| anyhow!("Error locking audio file mutex!"))?;
            audio_file.file.absolute_path.clone()
        };
        let fingerprint = AcoustIdService::instance().get_fingerprint(&file_path)?;

        // acoustid
        let lookup = AcoustIdService::instance()
            .lookup_fingerprint(&fingerprint)
            .await?;

        // recording
        let recordingid = lookup[0].recordings[0].id.clone();
        {
            let mut audio_file = audio_file
                .lock()
                .map_err(|_| anyhow!("Error locking audio file mutex!"))?;
            audio_file.acoust_id = Some(lookup[0].id.clone());
            audio_file.recording_id = Some(recordingid.clone());
        }

        // lookup
        let lookup = MusicBrainzService::instance()
            .lookup_releases_for_recording(&recordingid)
            .await?;

        // get album
        let albums = self.load_lookup(&lookup);
        let album = self.choose_album(&albums)?;
        {
            let mut album = album.lock().unwrap();
            album.match_file(audio_file);
            album.update_tag_fields(audio_file);
        }
        Ok(album)
    }

    pub fn load_lookup(&self, lookup: &ReleaseLookup) -> Vec<Am<Album>> {
        lookup
            .releases
            .iter()
            .map(|r| self.load_release(r))
            .collect()
    }

    pub fn load_release(&self, release: &Release) -> Am<Album> {
        let album = Album::from(release);
        let album_id = album.id.clone();
        let album = Arc::new(Mutex::new(album));
        self.albums.lock().unwrap().insert(album_id, album.clone());
        album
    }

    fn choose_album(&self, albums: &[Am<Album>]) -> Result<Am<Album>> {
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
}

impl Default for AlbumService {
    fn default() -> Self {
        Self::new()
    }
}
