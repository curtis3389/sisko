use crate::domain::{File, IFileService, ITrackService, TagField, Track};
use crate::ui::Ui;
use std::sync::{Arc, Mutex};
use syrette::injectable;
use syrette::ptr::SingletonPtr;

/// Represents a service for application actions.
/// This doesn't have a well-defined repsonsibility and should probably be refactored.
pub trait ISiskoService {
    /// Adds files in the given folder to sisko.
    ///
    /// # Arguments
    ///
    /// * `file` - The folder to add.
    fn add_folder(&self, file: Arc<File>);

    /// Selects the given track.
    /// This changes fields shown in the metadata table.
    ///
    /// # Arguments
    ///
    /// * `track` - The track to select.
    fn select_track(&self, track: &Arc<Mutex<Track>>);

    fn update_tag_field(&self, field: &TagField);
}

/// Represents a service for application actions.
pub struct SiskoService {
    /// A service for files.
    file_service: SingletonPtr<dyn IFileService>,

    /// A service for tracks.
    track_service: SingletonPtr<dyn ITrackService>,

    /// A service for the UI.
    ui: SingletonPtr<dyn Ui>,
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
        ui: SingletonPtr<dyn Ui>,
    ) -> Self {
        SiskoService {
            file_service,
            track_service,
            ui,
        }
    }
}

impl ISiskoService for SiskoService {
    fn add_folder(&self, file: Arc<File>) {
        let files = self.file_service.get_files_in_dir_recursive(&file.path);
        let tracks: Vec<Arc<Mutex<Track>>> =
            files.iter().map(|f| self.track_service.load(f)).collect();
        tracks.into_iter().for_each(|t| self.ui.add_cluster_file(t));
    }

    fn select_track(&self, track: &Arc<Mutex<Track>>) {
        self.ui.set_metadata_table(&track);
    }

    fn update_tag_field(&self, _field: &TagField) {
        //self.tag_service.update_tag_field(field);
        todo!()
    }
}
