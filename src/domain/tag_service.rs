use crate::domain::{File, Tag};
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::sync::OnceLock;

/// Represents a service for working with audio file tags.
pub struct TagService {}

impl TagService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<TagService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new tag service.
    pub fn new() -> Self {
        TagService {}
    }

    pub fn get_all(&self, file: &File) -> Vec<Tag> {
        let mut tags = vec![];
        if let Ok(id3v2) = ID3v2Tag::read_from_path(&file.absolute_path) {
            tags.push(Tag::from(&id3v2));
        }
        tags
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
    }
}
