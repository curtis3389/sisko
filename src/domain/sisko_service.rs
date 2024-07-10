use crate::domain::{File, IFileService, ISiskoService, ITrackService, Track};
use crate::ui::IUi;
use anyhow::Result;
use itertools::Itertools;
use std::sync::{Arc, Mutex};
use syrette::injectable;
use syrette::ptr::SingletonPtr;

/// Represents a service for application actions.
pub struct SiskoService {
    /// A service for files.
    file_service: SingletonPtr<dyn IFileService>,

    /// A service for tracks.
    track_service: SingletonPtr<dyn ITrackService>,

    /// A service for the UI.
    ui: SingletonPtr<dyn IUi>,
}

#[injectable(ISiskoService)]
impl SiskoService {
    /// Returns a new service for application actions.
    ///
    /// # Arguments
    ///
    /// * `file_service` - A service for files.
    /// * `track_service` - A service for tracks.
    /// * `ui` - A service for the UI.
    pub fn new(
        file_service: SingletonPtr<dyn IFileService>,
        track_service: SingletonPtr<dyn ITrackService>,
        ui: SingletonPtr<dyn IUi>,
    ) -> Self {
        SiskoService {
            file_service,
            track_service,
            ui,
        }
    }
}

impl ISiskoService for SiskoService {
    fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = self.file_service.get_files_in_dir_recursive(&file.path)?;
        let tracks: Vec<Arc<Mutex<Track>>> = files
            .iter()
            .map(|f| self.track_service.load(f))
            .try_collect()?;
        tracks.into_iter().for_each(|t| self.ui.add_cluster_file(t));
        Ok(())
    }

    fn select_track(&self, track: &Arc<Mutex<Track>>) {
        self.ui.set_metadata_table(track);
    }
}
