use crate::infrastructure::file::FileType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Represents a file that the program has looked at.
#[derive(Clone, Debug)]
pub struct File {
    /// The absolute path of the file.
    pub absolute_path: PathBuf,
    /// The name of the file.
    pub name: String,
    /// The size of the file.
    pub size: Option<u64>,
    /// The type of the file.
    pub file_type: Option<FileType>,
    /// The date the file was last modified.
    pub date_modified: Option<DateTime<Utc>>,
}
