use crate::infrastructure::Value;

/// Represents a column for a row in a table of tags.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TagColumn {}

impl TagColumn {
    /// Returns the display string for this tag column.
    pub fn as_str(&self) -> &str {
        ""
    }
}

impl Value for TagColumn {}
