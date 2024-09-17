use super::{AudioFileId, TagField, TagFieldId, TagFieldType, TagType};
use crate::{
    domain::events::DomainEvent,
    infrastructure::{Entity, EntityId},
};
use sisko_lib::{
    id3v2_header::ID3v2Header, id3v2_header_flags::ID3v2HeaderFlags, id3v2_tag::ID3v2Tag,
    id3v2_version_number::ID3v2VersionNumber,
};
use std::cmp::{Eq, PartialEq};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagId {
    pub audio_file_id: AudioFileId,
    pub tag_type: TagType,
}

impl TagId {
    pub fn new(audio_file_id: AudioFileId, tag_type: TagType) -> Self {
        Self {
            audio_file_id,
            tag_type,
        }
    }
}

impl EntityId for TagId {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<&TagId> for String {
    fn from(id: &TagId) -> Self {
        format!("{}:{}", id.audio_file_id.to_string(), id.tag_type)
    }
}

impl From<TagId> for String {
    fn from(id: TagId) -> Self {
        String::from(&id)
    }
}

/// Represents a metadata tag in an audio file.
#[derive(Clone, Debug)]
pub struct Tag {
    pub events: Vec<DomainEvent>,

    pub id: TagId,

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
    pub fn new(id: TagId, fields: Vec<TagField>) -> Self {
        Self {
            events: vec![],
            id,
            fields,
        }
    }

    /// Returns the value of the artist field, if there is one.
    pub fn artist(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(
                    TagFieldId {
                        tag_id: _,
                        tag_field_type: TagFieldType::Artist,
                    },
                    value,
                    _,
                ) => Some(value.clone()),
                _ => None,
            })
            .next()
    }

    pub fn get_field(&self, field_type: TagFieldType) -> Option<&TagField> {
        self.fields
            .iter()
            .find(|field| field.tag_field_type() == field_type)
    }

    pub fn has_changes(&self) -> bool {
        self.fields.iter().any(|field| field.has_new_value())
    }

    pub fn set_new_text_value(&mut self, field_id: TagFieldId, new_value: String) {
        if let TagField::Text(t, v, _) = match self.get_field(field_id.tag_field_type.clone()) {
            Some(field) => field.clone(),
            None => TagField::Text(field_id.clone(), String::new(), None),
        } {
            let field = TagField::Text(t, v, Some(new_value));
            self.update_field(field);
        }
    }

    /// Returns the value of the track title field, if there is one.
    pub fn title(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(
                    TagFieldId {
                        tag_id: _,
                        tag_field_type: TagFieldType::Title,
                    },
                    value,
                    _,
                ) => Some(value.clone()),
                _ => None,
            })
            .next()
    }

    /// Updates a field with the given new field data.
    /// This will update the field with the same type as the given field.
    pub fn update_field(&mut self, tag_field: TagField) {
        if let Some((index, _)) = self
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.tag_field_type() == tag_field.tag_field_type())
        {
            self.fields.remove(index);
            self.fields.insert(index, tag_field);
        } else {
            self.fields.push(tag_field);
        }
        self.events.push(DomainEvent::TagUpdated(self.clone()))
    }
}

impl Entity for Tag {
    type Id = TagId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: crate::infrastructure::EntityId,
    {
        todo!()
    }
}

impl Eq for Tag {}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.fields == other.fields
    }
}

pub trait Tags {
    fn artist(&self) -> Option<String>;
    fn title(&self) -> Option<String>;
}

impl Tags for Vec<Tag> {
    fn artist(&self) -> Option<String> {
        self.iter()
            .filter_map(|tag| {
                tag.fields
                    .iter()
                    .find(|field| field.tag_field_type() == TagFieldType::Artist)
                    .map(|field| field.display_value())
            })
            .next()
    }

    fn title(&self) -> Option<String> {
        self.iter()
            .filter_map(|tag| {
                tag.fields
                    .iter()
                    .find(|field| field.tag_field_type() == TagFieldType::Title)
                    .map(|field| field.display_value())
            })
            .next()
    }
}

impl From<&Tag> for ID3v2Tag {
    fn from(value: &Tag) -> Self {
        let frames = TagField::convert(&value.fields);
        let header = ID3v2Header {
            file_identifier: String::from("ID3"),
            version: ID3v2VersionNumber::new(4, 0),
            flags: ID3v2HeaderFlags {
                unsynchronisation: false,
                has_extended_header: false,
                is_experimental: false,
                has_footer: false,
            },
            size: frames.iter().map(|frame| frame.header.size).sum(),
        };
        ID3v2Tag {
            header,
            // TODO: save extended header with restriction settings
            extended_header: None,
            frames,
            // padding and footer determined when saving
            padding: 0,
            footer: None,
        }
    }
}
