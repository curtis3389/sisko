use crate::domain::{TagField, Track};

/// Represents the UI.
pub trait Ui {
    /// Adds the given track to the cluster file table.
    ///
    /// # Arguments
    ///
    /// * `track` - The track to add to the cluster file table.
    fn add_cluster_file(&self, track: Track);

    /// Opens the add directory dialog.
    fn open_directory_dialog(&self);

    /// Sets the tag fields in the metadata table.
    ///
    /// # Arguments
    ///
    /// * `fields` - The tag fields to show in the metadata table.
    fn set_metadata_table(&self, fields: &Vec<TagField>);
}
