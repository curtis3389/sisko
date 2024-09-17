use crate::infrastructure::Value;

/// Represents the possible columns in a table of audio files.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AudioFileColumn {
    /// The audio title column.
    Title,
    /// The audio artist column.
    Artist,
    /// The audio length column.
    Length,
}

impl AudioFileColumn {
    /// Returns the display string for this audio file column.
    pub fn as_str(&self) -> &str {
        match *self {
            AudioFileColumn::Title => "Title",
            AudioFileColumn::Artist => "Artist",
            AudioFileColumn::Length => "Length",
        }
    }
}

impl Value for AudioFileColumn {}
