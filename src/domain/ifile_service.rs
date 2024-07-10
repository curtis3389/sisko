use crate::domain::File;
use crate::ui::FileDialogType;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Represents a service for working with files.
pub trait IFileService {
    /// Gets the file with the given id..
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the file to get.
    fn get(&self, id: &Uuid) -> Result<Arc<File>>;

    /// Returns a vector of files under the given path for the given file/folder
    /// dialog type.
    ///
    /// This will also include a ".." file for the parent directory.
    ///
    /// If for a directory dialog, this only returns directories.
    /// If for an audio file dialog, this returns audio files and directories.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to get the files from.
    /// * `dialog_type` - The type of the file/folder dialog to get the file for.
    fn get_files_in_dir(&self, path: &Path, dialog_type: FileDialogType) -> Result<Vec<Arc<File>>>;

    /// Returns a vector of tracks found under the directory with the given path.
    /// This will search recursively.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to search for tracks.
    fn get_files_in_dir_recursive(&self, path: &Path) -> Result<Vec<Arc<File>>>;

    /// Loads and returns the file at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to load.
    fn load(&self, path: &Path) -> Result<Arc<File>>;
}
