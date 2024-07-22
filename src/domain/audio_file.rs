use crate::domain::{File, Tag, TagField, TagType};

/// Represents a file that contains audio data recognized by sisko.
#[derive(Clone, Debug)]
pub struct AudioFile {
    /// The file that contains audio data.
    pub file: File,

    /// The metadata tags on the file.
    pub tags: Vec<Tag>,
}

impl AudioFile {
    /// Returns a new audio file for the given file and tags.
    ///
    /// # Arguments
    ///
    /// * `file` - The file that contains audio data.
    /// * `tags` - The metadata tags from the file.
    pub fn new(file: File, tags: Vec<Tag>) -> Self {
        Self { file, tags }
    }

    /// Returns the artist of the audio, if any.
    pub fn artist(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.artist()).next()
    }

    /// Returns the length of the audio, if any.
    pub fn length(&self) -> Option<String> {
        None
    }

    /// Returns the title of the title, if any.
    pub fn title(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.title()).next()
    }

    /// Updates the field of a tag for the given tag type and field data.
    ///
    /// # Arguments
    ///
    /// * `tag_type` - The type of the tag to update the field of.
    /// * `tag_field` - The tag field data to update with.
    pub fn update_tag_field(&mut self, tag_type: &TagType, tag_field: TagField) {
        let tag = self
            .tags
            .iter_mut()
            .find(|t| t.tag_type == *tag_type)
            .unwrap_or_else(|| {
                panic!(
                    "Error trying to update {} tag that's not in the audio file!",
                    tag_type
                )
            });
        tag.update_field(tag_field);
    }
}
