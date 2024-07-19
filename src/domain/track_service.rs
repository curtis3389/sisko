use crate::domain::{File, TagService, Track};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

/// Represents a service for working with tracks.
/// A track is an audio file.
pub struct TrackService {
    /// The tracks loaded into the service.
    tracks: Mutex<HashMap<PathBuf, Arc<Mutex<Track>>>>,
}

impl TrackService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<TrackService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new track service.
    ///
    /// # Arguments
    ///
    /// * `tag_service` - A service for working with tags.
    pub fn new() -> Self {
        TrackService {
            tracks: Mutex::new(HashMap::new()),
        }
    }

    fn get_clone(&self, file: &File) -> Result<Arc<Mutex<Track>>> {
        Ok(self
            .tracks
            .lock()
            .map_err(|_| anyhow!("Failed to lock tracks mutex!"))?
            .get(&file.absolute_path)
            .ok_or(anyhow!(
                "File not found in tracks: {}!",
                file.absolute_path.to_string_lossy()
            ))?
            .clone())
    }

    fn is_loaded(&self, file: &File) -> Result<bool> {
        Ok(self
            .tracks
            .lock()
            .map_err(|_| anyhow!("Error locking tracks mutex!"))?
            .contains_key(&file.absolute_path))
    }

    fn load(&self, file: &File) -> Result<()> {
        let tags = TagService::instance().get_all(file);
        let track = Track::new(file.clone(), tags);
        let track = Arc::new(Mutex::new(track));
        self.tracks
            .lock()
            .map_err(|_| anyhow!("Failed to lock tracks mutex!"))?
            .insert(file.absolute_path.clone(), track.clone());
        Ok(())
    }

    pub fn get(&self, file: &File) -> Result<Arc<Mutex<Track>>> {
        if !self.is_loaded(file)? {
            self.load(file)?;
        }
        self.get_clone(file)
    }
}

impl Default for TrackService {
    fn default() -> Self {
        Self::new()
    }
}
