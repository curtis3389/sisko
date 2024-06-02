use crate::file::File;
use crate::file_dialog_type::FileDialogType;
use crate::file_type::FileType;
use chrono::DateTime;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use syrette::injectable;

/// Represents a service for working with files.
pub trait IFileService {
    /// Gets the file at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to get.
    fn get(&self, path: &PathBuf) -> File;

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
    fn get_files_in_dir(&self, path: &PathBuf, dialog_type: FileDialogType) -> Vec<File>;

    /// Returns a vector of tracks found under the directory with the given path.
    /// This will search recursively.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to search for tracks.
    fn get_files_in_dir_recursive(&self, path: &PathBuf) -> Vec<File>;
}

/// Represents a service for working with files backed by the native filesystem.
pub struct FileService {}

#[injectable(IFileService)]
impl FileService {
    /// Returns a new file service.
    pub fn new() -> Self {
        FileService {}
    }
}

impl IFileService for FileService {
    fn get(&self, path: &PathBuf) -> File {
        let metadata = path.metadata().ok();
        let file_type = Some(FileType::from(path));
        File {
            name: path
                .file_name()
                .map(|name| name.to_os_string())
                .expect("Every file looked at should have a filename!")
                .into_string()
                .unwrap_or(String::from("<invalid unicode>")),
            size: match &metadata {
                Some(metadata) => Some(metadata.len()),
                None => None,
            },
            file_type,
            date_modified: match &metadata {
                Some(metadata) => match metadata.modified() {
                    Ok(system_time) => match system_time.duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(duration) => DateTime::from_timestamp(
                            duration.as_secs() as i64,
                            duration.subsec_nanos(),
                        ),
                        Err(_) => None,
                    },
                    Err(_) => None,
                },
                None => None,
            },
            path: path.clone(),
        }
    }

    fn get_files_in_dir(&self, path: &PathBuf, dialog_type: FileDialogType) -> Vec<File> {
        let paths = fs::read_dir(path).unwrap();
        let mut files: Vec<File> = paths
            .filter_map(|dir_result| {
                let dir = dir_result.unwrap();
                let file = self.get(&dir.path());
                match file.file_type {
                    Some(file_type) => match dialog_type {
                        FileDialogType::AudioFile => match file_type {
                            FileType::UnsupportedFile => None,
                            _ => Some(file),
                        },
                        FileDialogType::Directory => match file_type {
                            FileType::Directory => Some(file),
                            _ => None,
                        },
                    },
                    None => None,
                }
            })
            .collect();

        if let Some(parent_path) = path.parent() {
            files.insert(
                0,
                File {
                    name: "..".to_string(),
                    size: None,
                    file_type: Some(FileType::Directory),
                    date_modified: None,
                    path: parent_path.to_path_buf(),
                },
            );
        }

        files
    }

    fn get_files_in_dir_recursive(&self, path: &PathBuf) -> Vec<File> {
        let files = self.get_files_in_dir(path, FileDialogType::AudioFile);
        let mut audio_files: Vec<File> = files
            .iter()
            .filter(|f| match f.file_type {
                Some(file_type) => file_type != FileType::Directory,
                None => false,
            })
            .map(|f| f.clone())
            .collect();
        let directories = files
            .iter()
            .filter(|f| match f.file_type {
                Some(file_type) => file_type == FileType::Directory,
                None => false,
            })
            .filter(|f| f.name != "..");
        let mut sub_files: Vec<File> = directories
            .map(|f| self.get_files_in_dir_recursive(&f.path))
            .flatten()
            .collect();
        audio_files.append(&mut sub_files);
        audio_files
    }
}
