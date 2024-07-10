use crate::domain::File;
use crate::ui::FileColumn;
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::Arc;

/// Represents the UI view of a file.
#[derive(Clone, Debug)]
pub struct FileView {
    /// The absolute path of the file.
    pub absolute_path: PathBuf,
    /// The name of the file.
    pub name: String,
    /// The size of the file.
    pub size: String,
    /// The type of the file.
    pub file_type: String,
    /// The date the file was last modified.
    pub date_modified: String,
    /// The file's path.
    pub path: String,
    /// The file.
    pub file: Arc<File>,
}

impl From<&Arc<File>> for FileView {
    fn from(file: &Arc<File>) -> FileView {
        FileView {
            absolute_path: file.absolute_path.clone(),
            name: file.name.clone(),
            size: match file.size {
                Some(size) => size.to_string(),
                None => String::new(),
            },
            file_type: match file.file_type {
                Some(file_type) => file_type.as_str().to_string(),
                None => String::from("Unknown"),
            },
            date_modified: file
                .date_modified
                .map_or(String::new(), |date| date.to_string()),
            path: String::from(file.path.to_string_lossy()),
            file: file.clone(),
        }
    }
}

impl TableViewItem<FileColumn> for FileView {
    /// Returns the value of the given column for this File.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    ///
    /// TODO: make sure modified date is formatted correctly
    fn to_column(&self, column: FileColumn) -> String {
        match column {
            FileColumn::Name => self.name.clone(),
            FileColumn::Size => self.size.clone(),
            FileColumn::Type => self.file_type.clone(),
            FileColumn::DateModified => self.date_modified.clone(),
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
