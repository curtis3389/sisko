use crate::domain::{File, Track};
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Represents a service for application actions.
/// This doesn't have a well-defined repsonsibility and should probably be refactored.
pub trait ISiskoService {
    /// Adds files in the given folder to sisko.
    ///
    /// # Arguments
    ///
    /// * `file` - The folder to add.
    fn add_folder(&self, file: Arc<File>) -> Result<()>;

    /// Selects the given track.
    /// This changes fields shown in the metadata table.
    ///
    /// # Arguments
    ///
    /// * `track` - The track to select.
    fn select_track(&self, track: &Arc<Mutex<Track>>);
}
