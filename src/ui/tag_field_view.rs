use crate::domain::{TagField, TagFieldType, TagType, Track};
use crate::ui::TagFieldColumn;
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

/// Represents the UI view of a tag field.
#[derive(Clone, Debug)]
pub struct TagFieldView {
    /// The track the tag field is from.
    pub track: Arc<Mutex<Track>>,

    /// The type of the tag the tag field is from.
    pub tag_type: TagType,

    /// The type of the tag field.
    pub tag_field_type: TagFieldType,
}

impl TagFieldView {
    /// Returns a new view of a tag field for the given track and types.
    ///
    /// # Arguments
    ///
    /// * `track` - The track the tag field is in.
    /// * `tag_type` - The type of the tag the field is in.
    /// * `field` - The type of the field.
    pub fn new(track: &Arc<Mutex<Track>>, tag_type: &TagType, field: &TagFieldType) -> Self {
        Self {
            track: track.clone(),
            tag_type: tag_type.clone(),
            tag_field_type: field.clone(),
        }
    }

    /// Returns the tag field referred to by this view.
    pub fn get_field(&self) -> TagField {
        let track = self.track.lock().expect("Error locking a track mutex!");
        let tag = track
            .tags
            .iter()
            .find(|tag| tag.tag_type == self.tag_type)
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't find {} tag in {}!",
                    self.tag_type,
                    track.file.absolute_path.to_string_lossy()
                )
            });
        tag.fields
            .iter()
            .find(|f| f.tag_field_type() == self.tag_field_type)
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't find {} field in {} tag in {}!",
                    self.tag_field_type,
                    self.tag_type,
                    track.file.absolute_path.to_string_lossy()
                )
            })
            .clone()
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
            TagFieldColumn::Tag => self.get_field().display_name(),
            TagFieldColumn::OriginalValue => self.get_field().display_value(),
            TagFieldColumn::NewValue => self.get_field().display_new_value(),
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
        let field = self.get_field();
        let other = other.get_field();
        match column {
            TagFieldColumn::Tag => field.display_name().cmp(&other.display_name()),
            TagFieldColumn::OriginalValue => field.display_value().cmp(&other.display_value()),
            TagFieldColumn::NewValue => field.display_new_value().cmp(&other.display_new_value()),
        }
    }
}
