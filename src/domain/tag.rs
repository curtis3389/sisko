use sisko_lib::id3v2_tag::ID3v2Tag;

use crate::domain::{TagField, TagFieldType, TagType};

/// Represents a metadata tag in an audio file.
#[derive(Clone, Debug)]
pub struct Tag {
    /// The type of the metadata tag.
    pub tag_type: TagType,

    /// The fields in the tag.
    pub fields: Vec<TagField>,
}

impl Tag {
    /// Returns a new tag with the given type and fields.
    ///
    /// # Arguments
    ///
    /// * `tag_type` - The type of the new tag.
    /// * `fields` - The fields in the new tag.
    pub fn new(tag_type: TagType, fields: Vec<TagField>) -> Self {
        Self { tag_type, fields }
    }

    /// Returns the value of the artist field, if there is one.
    pub fn artist(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(TagFieldType::Artist, value, _) => Some(value.clone()),
                _ => None,
            })
            .next()
    }

    /// Returns the value of the track title field, if there is one.
    pub fn title(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(TagFieldType::Title, value, _) => Some(value.clone()),
                _ => None,
            })
            .next()
    }

    /// Updates a field with the given new field data.
    /// This will update the field with the same type as the given field.
    pub fn update_field(&mut self, tag_field: TagField) {
        let (index, _) = self
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.tag_field_type() == tag_field.tag_field_type())
            .unwrap_or_else(|| {
                panic!(
                    "Error trying to update tag field {} that's not in the tag!",
                    tag_field.tag_field_type()
                )
            });
        self.fields.remove(index);
        self.fields.insert(index, tag_field);
    }
}

impl From<&ID3v2Tag> for Tag {
    fn from(id3v2: &ID3v2Tag) -> Self {
        Tag::new(TagType::ID3v2, TagField::parse_all(&id3v2.frames))
    }
}
