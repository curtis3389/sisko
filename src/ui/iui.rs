use crate::domain::Track;
use crate::ui::{TagFieldView, TrackView};
use std::sync::{Arc, Mutex};

/// Represents the UI.
pub trait IUi {
    /// Adds the given track to the cluster file table.
    ///
    /// # Arguments
    ///
    /// * `track` - The track to add to the cluster file table.
    fn add_cluster_file(&self, track: Arc<Mutex<Track>>);

    /// Opens the add directory dialog.
    fn open_directory_dialog(&self);

    fn open_logs(&self);

    /// Opens tag field details dialog.
    fn open_tag_field_dialog(&self, field: &TagFieldView);

    /// Opens the track actions dialog.
    fn open_track_dialog(&self, track: &TrackView);

    /// Sets the tag fields in the metadata table.
    ///
    /// # Arguments
    ///
    /// * `fields` - The tag fields to show in the metadata table.
    fn set_metadata_table(&self, track: &Arc<Mutex<Track>>);
}
