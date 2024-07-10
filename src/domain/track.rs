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
    /// Returns a new track for the given file and tags.
    ///
    /// # Arguments
    ///
    /// * `file` - The file the track is for.
    /// * `tags` - The metadata tags from the file.
    pub fn new(file: File, tags: Vec<Tag>) -> Self {
        Self { file, tags }
    }

    /// Returns the artist of the track, if any.
    pub fn artist(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.artist()).next()
    }

    /// Returns the length of the track, if any.
    pub fn length(&self) -> Option<String> {
        None
    }

    /// Returns the title of the track, if any.
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
                    "Error trying to update {} tag that's not in the track!",
                    tag_type
                )
            });
        tag.update_field(tag_field);
    }
}
