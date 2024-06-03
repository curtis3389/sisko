/// Represents a column for a row in a table of tags.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TagColumn {}

impl TagColumn {
    /// Returns the display string for this tag column.
    pub fn as_str(&self) -> &str {
        ""
    }
}
