use crate::file::File;
use crate::tag::{ID3v2TagWrapper, ITag};
use sisko_lib::id3v2_tag::ID3v2Tag;
use syrette::injectable;

/// Represents a service for working with tags.
pub trait ITagService {
    /// Gets the tags for the given track.
    ///
    /// # Arguments
    ///
    /// * `file` - The track to get the tags for.
    fn get(&self, file: &File) -> Vec<Box<dyn ITag>>;
}

/// Represents a service for working with audio file tags.
pub struct TagService {}

#[injectable(ITagService)]
impl TagService {
    /// Returns a new tag service.
    pub fn new() -> Self {
        TagService {}
    }
}

impl ITagService for TagService {
    fn get(&self, file: &File) -> Vec<Box<dyn ITag>> {
        let id3v2 = ID3v2Tag::read_from_path(&file.path).unwrap();
        vec![Box::new(ID3v2TagWrapper::new(id3v2))]
    }
}
