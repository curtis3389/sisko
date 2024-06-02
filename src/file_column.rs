/// Represents the possible columns in a folder/file table.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FileColumn {
    /// The file name column.
    Name,
    /// The file size column.
    Size,
    /// The file type column.
    Type,
    /// The date modified column.
    DateModified,
}

impl FileColumn {
    /// Returns the display string for this file column.
    pub fn as_str(&self) -> &str {
        match *self {
            FileColumn::Name => "Name",
            FileColumn::Size => "Size",
            FileColumn::Type => "Type",
            FileColumn::DateModified => "Date Modified",
        }
    }
}
