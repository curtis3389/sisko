use crate::domain::{File, ITagService, Tag};
use sisko_lib::id3v2_tag::ID3v2Tag;
use syrette::injectable;

/// Represents a service for working with audio file tags.
pub struct TagService {}

#[injectable(ITagService)]
impl TagService {
    /// Returns a new tag service.
    pub fn new() -> Self {
        TagService {}
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
    }
}

impl ITagService for TagService {
    fn get_all(&self, file: &File) -> Vec<Tag> {
        let mut tags = vec![];
        if let Ok(id3v2) = ID3v2Tag::read_from_path(&file.path) {
            tags.push(Tag::from(&id3v2));
        }
        tags
    }
}
