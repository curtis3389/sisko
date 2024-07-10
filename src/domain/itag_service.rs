use crate::domain::{File, Tag};

/// Represents a service for working with tags.
pub trait ITagService {
    /// Returns the tags in the given file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to get the tags for.
    fn get_all(&self, file: &File) -> Vec<Tag>;
}
