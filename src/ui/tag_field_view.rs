use crate::domain::{AudioFile, TagField, TagFieldType, TagType};
use crate::ui::TagFieldColumn;
use anyhow::{anyhow, Result};
use cursive_table_view::TableViewItem;
use log::error;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

/// Represents the UI view of a tag field.
#[derive(Clone, Debug)]
pub struct TagFieldView {
    /// The audio file the tag field is from.
    pub audio_file: Arc<Mutex<AudioFile>>,

    /// The type of the tag the tag field is from.
    pub tag_type: TagType,

    /// The type of the tag field.
    pub tag_field_type: TagFieldType,
}

impl TagFieldView {
    /// Returns a new view of a tag field for the given audio file and types.
    ///
    /// # Arguments
    ///
    /// * `audio_file` - The audio file the tag field is in.
    /// * `tag_type` - The type of the tag the field is in.
    /// * `field` - The type of the field.
    pub fn new(
        audio_file: &Arc<Mutex<AudioFile>>,
        tag_type: &TagType,
        field: &TagFieldType,
    ) -> Self {
        Self {
            audio_file: audio_file.clone(),
            tag_type: tag_type.clone(),
            tag_field_type: field.clone(),
        }
    }

    /// Returns the tag field referred to by this view.
    pub fn get_field(&self) -> Result<TagField> {
        let audio_file = self
            .audio_file
            .lock()
            .map_err(|_| anyhow!("Error locking audio files mutex!"))?;
        let tag = audio_file
            .tags
            .iter()
            .find(|tag| tag.tag_type == self.tag_type)
            .ok_or_else(|| {
                anyhow!(
                    "Couldn't find {} tag in {}!",
                    self.tag_type,
                    audio_file.file.absolute_path.to_string_lossy()
                )
            })?;
        Ok(tag
            .fields
            .iter()
            .find(|f| f.tag_field_type() == self.tag_field_type)
            .ok_or_else(|| {
                anyhow!(
                    "Couldn't find {} field in {} tag in {}!",
                    self.tag_field_type,
                    self.tag_type,
                    audio_file.file.absolute_path.to_string_lossy()
                )
            })?
            .clone())
    }
}

impl TableViewItem<TagFieldColumn> for TagFieldView {
    /// Returns the value of the given column for this Tag.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: TagFieldColumn) -> String {
        let s = match column {
            TagFieldColumn::Tag => self.get_field().map(|f| f.display_name()),
            TagFieldColumn::OriginalValue => self.get_field().map(|f| f.display_value()),
            TagFieldColumn::NewValue => self.get_field().map(|f| f.display_new_value()),
        };
        match s {
            Ok(s) => s,
            Err(e) => {
                // TODO: log error
                e.to_string()
            }
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
        let result = (|| -> Result<Ordering> {
            let field = self.get_field()?;
            let other = other.get_field()?;
            Ok(match column {
                TagFieldColumn::Tag => field.display_name().cmp(&other.display_name()),
                TagFieldColumn::OriginalValue => field.display_value().cmp(&other.display_value()),
                TagFieldColumn::NewValue => {
                    field.display_new_value().cmp(&other.display_new_value())
                }
            })
        })();
        match result {
            Ok(o) => o,
            Err(e) => {
                error!("Error comparing tag field views: {e}!");
                Ordering::Less
            }
        }
    }
}
