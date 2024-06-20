use crate::domain::{File, Tag, TagField, TagType};

/// Represents an audio track.
#[derive(Clone, Debug)]
pub struct Track {
    /// The file this track is from.
    pub file: File,

    /// The tags on the file.
    pub tags: Vec<Tag>,
}

impl Track {
    pub fn new(file: File, tags: Vec<Tag>) -> Self {
        Self { file, tags }
    }

    pub fn artist(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.artist()).next()
    }

    pub fn length(&self) -> Option<String> {
        None
    }

    pub fn title(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.title()).next()
    }

    pub fn update_tag_field(&mut self, tag_type: &TagType, tag_field: TagField) {
        let tag = self
            .tags
            .iter_mut()
            .filter(|t| t.tag_type == *tag_type)
            .next()
            .unwrap();
        tag.update_field(tag_field);
    }
}
