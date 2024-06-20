use crate::domain::{File, FileType};
use crate::ui::FileDialogType;
use chrono::DateTime;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use syrette::injectable;
use uuid::Uuid;

/// Represents a service for working with files.
pub trait IFileService {
    /// Gets the file at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to get.
    fn get(&self, id: &Uuid) -> Arc<File>;

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
    fn get_files_in_dir(&self, path: &PathBuf, dialog_type: FileDialogType) -> Vec<Arc<File>>;

    /// Returns a vector of tracks found under the directory with the given path.
    /// This will search recursively.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to search for tracks.
    fn get_files_in_dir_recursive(&self, path: &PathBuf) -> Vec<Arc<File>>;

    fn load(&self, path: &PathBuf) -> Arc<File>;
}

/// Represents a service for working with files backed by the native filesystem.
pub struct FileService {
    files: Mutex<HashMap<Uuid, Arc<File>>>,
    namespace_id: Uuid,
}

#[injectable(IFileService)]
impl FileService {
    /// Returns a new file service.
    pub fn new() -> Self {
        FileService {
            files: Mutex::new(HashMap::new()),
            namespace_id: Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"files.sisko.org"),
        }
    }
}

impl IFileService for FileService {
    fn get(&self, id: &Uuid) -> Arc<File> {
        self.files.lock().unwrap().get(id).unwrap().clone()
    }

    fn get_files_in_dir(&self, path: &PathBuf, dialog_type: FileDialogType) -> Vec<Arc<File>> {
        let paths = fs::read_dir(path).unwrap();
        let mut files: Vec<Arc<File>> = paths
            .filter_map(|dir_result| {
                let dir = dir_result.unwrap();
                let file = self.load(&dir.path());
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
            let path = parent_path.to_path_buf();
            let absolute_path = fs::canonicalize(&path).unwrap();
            files.insert(
                0,
                Arc::new(File {
                    id: Uuid::new_v5(
                        &self.namespace_id,
                        absolute_path.to_string_lossy().as_bytes(),
                    ),
                    absolute_path,
                    name: "..".to_string(),
                    size: None,
                    file_type: Some(FileType::Directory),
                    date_modified: None,
                    path,
                }),
            );
        }

        files
    }

    fn get_files_in_dir_recursive(&self, path: &PathBuf) -> Vec<Arc<File>> {
        let files = self.get_files_in_dir(path, FileDialogType::AudioFile);
        let mut audio_files: Vec<Arc<File>> = files
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
        let mut sub_files: Vec<Arc<File>> = directories
            .map(|f| self.get_files_in_dir_recursive(&f.path))
            .flatten()
            .collect();
        audio_files.append(&mut sub_files);
        audio_files
    }

    fn load(&self, path: &PathBuf) -> Arc<File> {
        let metadata = path.metadata().ok();
        let file_type = Some(FileType::from(path));
        let absolute_path = fs::canonicalize(path).unwrap();
        let file = File {
            id: Uuid::new_v5(
                &self.namespace_id,
                absolute_path.to_string_lossy().as_bytes(),
            ),
            absolute_path,
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
        };
        let file = Arc::new(file);
        let mut files = self.files.lock().unwrap();
        files.insert(file.id, file.clone());
        file
    }
}
