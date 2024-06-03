use crate::domain::TagField;
use crate::ui::TagFieldColumn;
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;

/// Represents the UI view of a tag field.
#[derive(Clone, Debug)]
pub struct TagFieldView {
    /// The name of the field.
    pub field_name: String,

    /// The original value of the field.
    pub field_value: String,

    /// The new value for the field.
    pub new_field_value: String,
}

impl From<&TagField> for TagFieldView {
    fn from(field: &TagField) -> Self {
        Self {
            field_name: field.field_name.clone(),
            field_value: field.field_value.clone(),
            new_field_value: match &field.new_field_value {
                Some(s) => s.clone(),
                None => String::new(),
            },
        }
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
            TagFieldColumn::Tag => self.field_name.to_string(),
            TagFieldColumn::OriginalValue => self.field_value.to_string(),
            TagFieldColumn::NewValue => self.new_field_value.to_string(),
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
            TagFieldColumn::Tag => self.field_name.cmp(&other.field_name),
            TagFieldColumn::OriginalValue => self.field_value.cmp(&other.field_value),
            TagFieldColumn::NewValue => self.new_field_value.cmp(&other.new_field_value),
        }
    }
}
