use super::TagFieldColumn;
use crate::domain::models::{TagField, TagFieldId};
use crate::infrastructure::{Entity, EntityId};
use cursive_table_view::TableViewItem;
use std::cmp::{Eq, Ordering, PartialEq};

/// Represents the UI view of a tag field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagFieldView {
    pub id: TagFieldId,

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
    pub fn new(field: &TagField) -> Self {
        Self {
            id: field.id().clone(),
            name: field.display_name(),
            new_value: field.display_new_value(),
            value: field.display_value(),
        }
    }
}

impl Entity for TagFieldView {
    type Id = TagFieldId;

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
