use crate::domain::FileType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct File {
    pub absolute_path: PathBuf,
    /// The name of the file.
    pub name: String,
    /// The size of the file.
    pub size: Option<u64>,
    /// The type of the file.
    pub file_type: Option<FileType>,
    /// The date the file was last modified.
    pub date_modified: Option<DateTime<Utc>>,
    /// The file's path.
    pub path: PathBuf,
}
