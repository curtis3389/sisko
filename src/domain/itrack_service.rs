use crate::domain::{File, Track};
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Represents a service for working with tracks (i.e. audio files).
pub trait ITrackService {
    /// Gets the track for the given file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to get the track for.
    fn get(&self, file: &File) -> Result<Arc<Mutex<Track>>>;

    /// Loads the given file as a track and returns it.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to load into the track service as a track.
    fn load(&self, file: &File) -> Result<Arc<Mutex<Track>>>;
}
