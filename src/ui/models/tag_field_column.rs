use crate::infrastructure::Value;

/// Represents the possible columns in a tag table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TagFieldColumn {
    /// The tag type.
    Tag,
    /// The original value of the tag.
    OriginalValue,
    /// The new value of the tag.
    NewValue,
}

impl TagFieldColumn {
    /// Returns the display string for this tag column.
    pub fn as_str(&self) -> &str {
        match *self {
            TagFieldColumn::Tag => "Tag",
            TagFieldColumn::OriginalValue => "Original Value",
            TagFieldColumn::NewValue => "New Value",
        }
    }
}

impl Value for TagFieldColumn {}
