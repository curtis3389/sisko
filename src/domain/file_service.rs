use crate::domain::{File, FileType, IFileService};
use crate::ui::FileDialogType;
use anyhow::{anyhow, Result};
use chrono::DateTime;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use syrette::injectable;
use uuid::Uuid;

/// Represents a service for working with files backed by the native filesystem.
pub struct FileService {
    /// The files loaded into the service.
    files: Mutex<HashMap<Uuid, Arc<File>>>,

    /// The ID of the file ID namespace.
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

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

impl IFileService for FileService {
    fn get(&self, id: &Uuid) -> Result<Arc<File>> {
        let file = self
            .files
            .lock()
            .map_err(|_| anyhow!("Failed to lock files mutex!"))?
            .get(id)
            .ok_or(anyhow!("Couldn't find file with the id {}!", id))?
            .clone();
        Ok(file)
    }

    fn get_files_in_dir(&self, path: &Path, dialog_type: FileDialogType) -> Result<Vec<Arc<File>>> {
        let paths = fs::read_dir(path)?;
        let files: Vec<Arc<File>> = paths
            .map(|dir_result| -> Result<Arc<File>> {
                let dir = dir_result?;
                self.load(&dir.path())
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
            let absolute_path = fs::canonicalize(&path)?;
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

        Ok(files)
    }

    fn get_files_in_dir_recursive(&self, path: &Path) -> Result<Vec<Arc<File>>> {
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
            .map(|f| self.get_files_in_dir_recursive(&f.path))
            .try_collect()?;
        let mut sub_files: Vec<Arc<File>> = sub_files.into_iter().flatten().collect();
        audio_files.append(&mut sub_files);
        Ok(audio_files)
    }

    fn load(&self, path: &Path) -> Result<Arc<File>> {
        let metadata = path.metadata().ok();
        let file_type = Some(FileType::from(path));
        let absolute_path = fs::canonicalize(path)?;
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
            path: path.to_path_buf(),
        };
        let file = Arc::new(file);
        let mut files = self
            .files
            .lock()
            .map_err(|_| anyhow!("Failed to lock the files mutex!"))?;
        files.insert(file.id, file.clone());
        Ok(file)
    }
}
