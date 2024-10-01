use super::TagFieldColumn;
use crate::{
    domain::models::{AudioFileId, FieldValueExtensions, MetadataField, TagFieldType},
    infrastructure::{Entity, EntityId},
};
use cursive_table_view::TableViewItem;
use std::cmp::{Eq, Ordering, PartialEq};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagFieldViewId {
    pub audio_file_id: AudioFileId,

    pub field_type: TagFieldType,
}

impl EntityId for TagFieldViewId {
    fn to_string(&self) -> String {
        format!(
            "{}:{}",
            self.audio_file_id.to_string(),
            self.field_type.display_name()
        )
    }
}

/// Represents the UI view of a tag field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagFieldView {
    pub id: TagFieldViewId,

    pub name: String,

    pub new_value: String,

    pub value: String,
}

impl TagFieldView {
    /// Returns a new view of a tag field for the given audio file and types.
    ///
    /// # Arguments
    ///
    /// * `field` - The type of the field.
    pub fn new(audio_file_id: &AudioFileId, field: &MetadataField) -> Self {
        Self {
            id: TagFieldViewId {
                audio_file_id: audio_file_id.clone(),
                field_type: field.field_type.clone(),
            },
            name: field.field_type.display_name(),
            new_value: field.new_value.display_value(),
            value: field.old_value.display_value(),
        }
    }
}

impl Entity for TagFieldView {
    type Id = TagFieldViewId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: EntityId,
    {
        &self.id
    }
}

impl TableViewItem<TagFieldColumn> for TagFieldView {
    /// Returns the value of the given column for this Tag.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: TagFieldColumn) -> String {
        match column {
            TagFieldColumn::Tag => self.name.clone(),
            TagFieldColumn::OriginalValue => self.value.clone(),
            TagFieldColumn::NewValue => self.new_value.clone(),
        }
    }

    /// Compares the value of the given column to another Tag.
    ///
    /// # Arguments
    ///
    /// * `other` - The other Tag to compare to.
    /// * `column` - The column to compare between the Tags.
    fn cmp(&self, other: &Self, column: TagFieldColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            TagFieldColumn::Tag => self.name.cmp(&other.name),
            TagFieldColumn::OriginalValue => self.value.cmp(&other.value),
            TagFieldColumn::NewValue => self.new_value.cmp(&other.new_value),
        }
    }
}
