use crate::domain::{TagField, TagFieldType, TagType};

/// Represents a metadata tag in an audio file.
#[derive(Clone, Debug)]
pub struct Tag {
    pub tag_type: TagType,
    pub fields: Vec<TagField>,
}

impl Tag {
    pub fn new(tag_type: TagType, fields: Vec<TagField>) -> Self {
        Self { tag_type, fields }
    }

    pub fn artist(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(tag_field_type, value, _) => match tag_field_type {
                    TagFieldType::Artist => Some(value.clone()),
                    _ => None,
                },
                _ => None,
            })
            .next()
    }

    pub fn title(&self) -> Option<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f {
                TagField::Text(tag_field_type, value, _) => match tag_field_type {
                    TagFieldType::Title => Some(value.clone()),
                    _ => None,
                },
                _ => None,
            })
            .next()
    }

    pub fn update_field(&mut self, tag_field: TagField) {
        let (index, _) = self
            .fields
            .iter()
            .enumerate()
            .filter(|(_, ref field)| field.tag_field_type() == tag_field.tag_field_type())
            .next()
            .unwrap();
        self.fields.remove(index);
        self.fields.insert(index, tag_field);
    }
}
