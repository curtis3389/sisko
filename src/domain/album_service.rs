use super::{Album, AudioFile, Recording};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::musicbrainz::MusicBrainzService;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct AlbumService {
    albums: Mutex<HashMap<String, Arc<Mutex<Album>>>>,
    recordings: Mutex<HashMap<String, Arc<Mutex<Recording>>>>,
}

impl AlbumService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AlbumService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            albums: Mutex::new(HashMap::new()),
            recordings: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_album(&self, album_id: &String) -> Result<Arc<Mutex<Album>>> {
        if !self.is_album_loaded(album_id)? {
            self.load_album(album_id).await?;
        }
        self.get_album_clone(album_id)
    }

    pub async fn get_album_for_recording(
        &self,
        recording: &Arc<Mutex<Recording>>,
    ) -> Result<Arc<Mutex<Album>>> {
        let album_id = self.get_album_id(recording).await?;
        self.get_album(&album_id).await
    }

    pub async fn get_recording(&self, recording_id: &String) -> Result<Arc<Mutex<Recording>>> {
        if !self.is_recording_loaded(recording_id)? {
            self.load_recording(recording_id).await?;
        }
        self.get_recording_clone(recording_id)
    }

    pub async fn get_recording_for_file(
        &self,
        file: &Arc<Mutex<AudioFile>>,
    ) -> Result<Arc<Mutex<Recording>>> {
        // fingerprint
        let file_path = {
            let file = file
                .lock()
                .map_err(|_| anyhow!("Error locking file mutex!"))?;
            file.file.absolute_path.clone()
        };
        let fingerprint = AcoustIdService::instance().get_fingerprint(&file_path)?;

        // acoustid
        let lookup = AcoustIdService::instance()
            .lookup_fingerprint(&fingerprint)
            .await?;

        // recording
        let recordingid = lookup[0].recordings[0].id.clone();
        let recording = self.get_recording(&recordingid).await?;

        {
            let audio_files = &mut recording.lock().unwrap().audio_files;
            if !audio_files
                .iter()
                .any(|t| t.lock().unwrap().file.absolute_path == file_path)
            {
                audio_files.push(file.clone());
            }
        }

        Ok(recording)
    }

    fn is_album_loaded(&self, album_id: &String) -> Result<bool> {
        Ok(self
            .albums
            .lock()
            .map_err(|_| anyhow!("Error locking albums mutex!"))?
            .contains_key(album_id))
    }

    fn is_recording_loaded(&self, recording_id: &String) -> Result<bool> {
        Ok(self
            .recordings
            .lock()
            .map_err(|_| anyhow!("Error locking recordings mutex!"))?
            .contains_key(recording_id))
    }

    async fn load_album(&self, album_id: &str) -> Result<()> {
        // releases
        let release = MusicBrainzService::instance()
            .lookup_release(album_id)
            .await?;

        // release details
        let mut recordings: Vec<Arc<Mutex<Recording>>> = vec![];
        for media in &release.media {
            for track in &media.tracks {
                let recording = self.get_recording(&track.recording.id).await?;
                recordings.push(recording);
            }
        }

        let album = Arc::new(Mutex::new(Album::new(release, recordings)));
        self.albums
            .lock()
            .map_err(|_| anyhow!("Error locking albums mutex!"))?
            .insert(album_id.to_string(), album);

        Ok(())
    }

    async fn load_recording(&self, recording_id: &str) -> Result<()> {
        let recording = MusicBrainzService::instance()
            .lookup_recording(recording_id)
            .await?;

        let recording = Arc::new(Mutex::new(Recording::from(&recording)));
        self.recordings
            .lock()
            .map_err(|_| anyhow!("Error locking recordings mutex!"))?
            .insert(recording_id.to_string(), recording);
        Ok(())
    }

    async fn get_album_id(&self, recording: &Arc<Mutex<Recording>>) -> Result<String> {
        let release_ids: Vec<String> = {
            recording
                .lock()
                .map_err(|_| anyhow!("Error locking recording mutex!"))?
                .release_ids
                .clone()
        };
        let mut albums: Vec<(Arc<Mutex<Album>>, f64)> = vec![];
        for id in release_ids {
            let album = self.get_album(&id).await?;
            let priority = self.get_priority(&album);
            albums.push((album, priority));
        }
        albums.sort_unstable_by(|(_, p1), (_, p2)| p1.total_cmp(p2));
        let (album, _) = &albums[0];
        let album = album
            .lock()
            .map_err(|_| anyhow!("Error locking album mutex!"))?;
        Ok(album.id.clone())
    }

    fn get_priority(&self, album: &Arc<Mutex<Album>>) -> f64 {
        0.5
    }

    fn get_album_clone(&self, album_id: &String) -> Result<Arc<Mutex<Album>>> {
        Ok(self
            .albums
            .lock()
            .map_err(|_| anyhow!("Error locking albums mutex!"))?
            .get(album_id)
            .ok_or(anyhow!("Error getting album with id {}", album_id))?
            .clone())
    }

    fn get_recording_clone(&self, recording_id: &String) -> Result<Arc<Mutex<Recording>>> {
        Ok(self
            .recordings
            .lock()
            .map_err(|_| anyhow!("Error locking recordings mutex!"))?
            .get(recording_id)
            .ok_or(anyhow!("Error getting recording with id {}", recording_id))?
            .clone())
    }
}

impl Default for AlbumService {
    fn default() -> Self {
        Self::new()
    }
}
