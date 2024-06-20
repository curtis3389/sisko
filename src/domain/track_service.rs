use crate::domain::{File, ITagService, Track};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use syrette::injectable;
use syrette::ptr::SingletonPtr;

/// Represents a service for working with tracks (i.e. audio files).
pub trait ITrackService {
    /// Gets the track for the given file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to load as a track.
    fn get(&self, file: &File) -> Arc<Mutex<Track>>;

    fn load(&self, file: &File) -> Arc<Mutex<Track>>;
}

/// Represents a service for working with tracks.
/// A track is an audio file.
pub struct TrackService {
    tag_service: SingletonPtr<dyn ITagService>,
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
}

impl ITrackService for TrackService {
    fn get(&self, file: &File) -> Arc<Mutex<Track>> {
        let tracks = self.tracks.lock().unwrap();
        tracks.get(&file.absolute_path).unwrap().clone()
    }

    fn load(&self, file: &File) -> Arc<Mutex<Track>> {
        let tags = self.tag_service.get_all(&file);
        let track = Track::new(file.clone(), tags);
        let track = Arc::new(Mutex::new(track));
        self.tracks
            .lock()
            .unwrap()
            .insert(file.absolute_path.clone(), track.clone());
        track
    }
}
