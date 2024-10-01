use crate::domain::models::Metadata;

/// Represents the UI view of a tag.
pub struct TagView {}

impl From<&Metadata> for TagView {
    fn from(_tag: &Metadata) -> Self {
        Self {}
    }
}
