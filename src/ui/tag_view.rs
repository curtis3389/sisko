use crate::domain::Tag;

/// Represents the UI view of a tag.
pub struct TagView {}

impl From<&Tag> for TagView {
    fn from(_tag: &Tag) -> Self {
        Self {}
    }
}
