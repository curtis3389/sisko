/// Represents the possible columns in a tag table.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TagColumn {
    /// The tag type.
    Tag,
    /// The original value of the tag.
    OriginalValue,
    /// The new value of the tag.
    NewValue,
}

impl TagColumn {
    /// Returns the display string for this tag column.
    pub fn as_str(&self) -> &str {
        match *self {
            TagColumn::Tag => "Tag",
            TagColumn::OriginalValue => "Original Value",
            TagColumn::NewValue => "New Value",
        }
    }
}
