use crate::domain::Tag;

/// Represents the UI view of a tag.
pub struct TagView {}

impl From<Box<dyn Tag>> for TagView {
    fn from(_tag: Box<dyn Tag>) -> Self {
        Self {}
    }
}
