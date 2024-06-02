use crate::file_column::FileColumn;
use crate::file_type::FileType;
use chrono::{DateTime, Utc};
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;
use std::path::PathBuf;

/// Represents a file.
#[derive(Clone, Debug)]
pub struct File {
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

impl TableViewItem<FileColumn> for File {
    /// Returns the value of the given column for this File.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    ///
    /// TODO: make sure modified date is formatted correctly
    fn to_column(&self, column: FileColumn) -> String {
        match column {
            FileColumn::Name => self.name.to_string(),
            FileColumn::Size => match self.size {
                Some(size) => size.to_string(),
                None => String::new(),
            },
            FileColumn::Type => match self.file_type {
                Some(file_type) => file_type.as_str().to_string(),
                None => String::from("Unknown"),
            },
            FileColumn::DateModified => self
                .date_modified
                .map_or(String::new(), |date| date.to_string()),
        }
    }

    /// Compares the value of the given column to another File.
    ///
    /// # Arguments
    ///
    /// * `other` - The other File to compare to.
    /// * `column` - The column to compare between the Files.
    fn cmp(&self, other: &Self, column: FileColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            FileColumn::Name => self.name.cmp(&other.name),
            FileColumn::Size => self.size.cmp(&other.size),
            FileColumn::Type => self.file_type.cmp(&other.file_type),
            FileColumn::DateModified => self.date_modified.cmp(&other.date_modified),
        }
    }
}
