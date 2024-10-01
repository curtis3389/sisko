use crate::infrastructure::file::{File, FileType};
use crate::ui::models::FileDialogType;
use anyhow::{anyhow, Result};
use chrono::DateTime;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

/// Represents a service for working with files backed by the native filesystem.
pub struct FileService {
    // TODO: remove this cache
    /// The files loaded into the service.
    files: Mutex<HashMap<PathBuf, Arc<File>>>,
}

impl FileService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<FileService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new file service.
    pub fn new() -> Self {
        FileService {
            files: Mutex::new(HashMap::new()),
        }
    }

    /// Returns a clone of the file with the given absolute path.
    fn get_clone(&self, path: &Path) -> Result<Arc<File>> {
        Ok(self
            .files
            .lock()
            .map_err(|_| anyhow!("Error locking files mutex!"))?
            .get(path)
            .ok_or(anyhow!(
                "Error getting file with path \"{}\"!",
                path.to_string_lossy()
            ))?
            .clone())
    }

    /// Returns whether the file with the given absolute path is loaded into the service.
    fn is_loaded(&self, path: &Path) -> Result<bool> {
        Ok(self
            .files
            .lock()
            .map_err(|_| anyhow!("Error locking files mutex!"))?
            .contains_key(path))
    }

    /// Loads the file with the given path into the service.
    fn load(&self, path: &Path) -> Result<()> {
        let metadata = path.metadata().ok();
        let file_type = Some(FileType::from(path));
        let file = File {
            absolute_path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|name| name.to_os_string())
                .ok_or_else(|| {
                    anyhow!(
                        "Error getting name for file at path {}!",
                        path.to_string_lossy()
                    )
                })?
                .into_string()
                .unwrap_or(String::from("<invalid unicode>")),
            size: metadata.as_ref().map(|metadata| metadata.len()),
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
        };
        let file = Arc::new(file);
        let mut files = self
            .files
            .lock()
            .map_err(|_| anyhow!("Failed to lock the files mutex!"))?;
        files.insert(file.absolute_path.clone(), file.clone());
        Ok(())
    }

    pub fn get(&self, path: &Path) -> Result<Arc<File>> {
        let path = fs::canonicalize(path)?;
        if !self.is_loaded(&path)? {
            self.load(&path)?;
        }
        self.get_clone(&path)
    }

    pub fn get_files_in_dir(
        &self,
        path: &Path,
        dialog_type: FileDialogType,
    ) -> Result<Vec<Arc<File>>> {
        let paths = fs::read_dir(path)?;
        let files: Vec<Arc<File>> = paths
            .map(|dir_result| -> Result<Arc<File>> {
                let dir = dir_result?;
                self.get(&dir.path())
            })
            .try_collect()?;
        let mut files: Vec<Arc<File>> = files
            .into_iter()
            .filter_map(|file| match file.file_type {
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
            })
            .collect();

        if let Some(parent_path) = path.parent() {
            let path = parent_path.to_path_buf();
            let absolute_path = fs::canonicalize(path)?;
            files.insert(
                0,
                Arc::new(File {
                    absolute_path,
                    name: "..".to_string(),
                    size: None,
                    file_type: Some(FileType::Directory),
                    date_modified: None,
                }),
            );
        }

        Ok(files)
    }

    pub fn get_files_in_dir_recursive(&self, path: &Path) -> Result<Vec<Arc<File>>> {
        let files = self.get_files_in_dir(path, FileDialogType::AudioFile)?;
        let mut audio_files: Vec<Arc<File>> = files
            .iter()
            .filter(|f| match f.file_type {
                Some(file_type) => file_type != FileType::Directory,
                None => false,
            })
            .cloned()
            .collect();
        let directories = files
            .iter()
            .filter(|f| match f.file_type {
                Some(file_type) => file_type == FileType::Directory,
                None => false,
            })
            .filter(|f| f.name != "..");
        let sub_files: Vec<Vec<Arc<File>>> = directories
            .map(|f| self.get_files_in_dir_recursive(&f.absolute_path))
            .try_collect()?;
        let mut sub_files: Vec<Arc<File>> = sub_files.into_iter().flatten().collect();
        audio_files.append(&mut sub_files);
        Ok(audio_files)
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}
