use crate::domain::{File, ITagService, ITrackService, Track};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use syrette::injectable;
use syrette::ptr::SingletonPtr;

/// Represents a service for working with tracks.
/// A track is an audio file.
pub struct TrackService {
    /// A tag service for getting the tags in files.
    tag_service: SingletonPtr<dyn ITagService>,

    /// The tracks loaded into the service.
    tracks: Mutex<HashMap<PathBuf, Arc<Mutex<Track>>>>,
}

#[injectable(ITrackService)]
impl TrackService {
    /// Returns a new track service.
    ///
    /// # Arguments
    ///
    /// * `tag_service` - A service for working with tags.
    pub fn new(tag_service: SingletonPtr<dyn ITagService>) -> Self {
        TrackService {
            tag_service,
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
        let tags = self.tag_service.get_all(file);
        let track = Track::new(file.clone(), tags);
        let track = Arc::new(Mutex::new(track));
        self.tracks
            .lock()
            .map_err(|_| anyhow!("Failed to lock tracks mutex!"))?
            .insert(file.absolute_path.clone(), track.clone());
        Ok(())
    }
}

impl ITrackService for TrackService {
    fn get(&self, file: &File) -> Result<Arc<Mutex<Track>>> {
        if !self.is_loaded(file)? {
            self.load(file)?;
        }
        self.get_clone(file)
    }
}
