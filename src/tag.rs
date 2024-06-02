use crate::tag_column::TagColumn;
use cursive_table_view::TableViewItem;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::cmp::Ordering;

/// Represents a metadata tag in an audio file.
pub trait ITag {
    /// Returns the artist field, if any.
    fn artist(&self) -> Option<String>;

    /// Returns the name of this tag (e.g. "ID3v2").
    fn tag_name(&self) -> String;

    /// Returns the track title field, if any.
    fn title(&self) -> Option<String>;
}

#[derive(Clone, Debug)]
pub struct ID3v2TagWrapper {
    tag: ID3v2Tag,
}

impl ID3v2TagWrapper {
    pub fn new(tag: ID3v2Tag) -> Self {
        ID3v2TagWrapper { tag }
    }

    fn get_text_field(&self, frame_id: &str) -> Option<String> {
        Some(
            self.tag
                .frames
                .iter()
                .filter(|f| f.header.frame_id == frame_id)
                .map(|f| match &f.fields {
                    ID3v2FrameFields::TextFields { encoding: _, text } => text[0].clone(),
                    _ => panic!(),
                })
                .collect::<Vec<String>>()
                .remove(0),
        )
    }
}

impl ITag for ID3v2TagWrapper {
    fn artist(&self) -> Option<String> {
        self.get_text_field("TPE1")
    }

    fn tag_name(&self) -> String {
        "ID3v2".to_string()
    }

    fn title(&self) -> Option<String> {
        self.get_text_field("TIT2")
    }
}

/// Represents a metadata tag in an audio file.
#[derive(Clone, Debug)]
pub struct Tag {
    /// The tag type.
    pub tag: String,
    /// The original value of the tag.
    pub original_value: String,
    /// The new value of the tag.
    pub new_value: String,
}

impl TableViewItem<TagColumn> for Tag {
    /// Returns the value of the given column for this Tag.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: TagColumn) -> String {
        match column {
            TagColumn::Tag => self.tag.to_string(),
            TagColumn::OriginalValue => self.original_value.to_string(),
            TagColumn::NewValue => self.new_value.to_string(),
        }
    }

    /// Compares the value of the given column to another Tag.
    ///
    /// # Arguments
    ///
    /// * `other` - The other Tag to compare to.
    /// * `column` - The column to compare between the Tags.
    fn cmp(&self, other: &Self, column: TagColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            TagColumn::Tag => self.tag.cmp(&other.tag),
            TagColumn::OriginalValue => self.original_value.cmp(&other.original_value),
            TagColumn::NewValue => self.new_value.cmp(&other.new_value),
        }
    }
}
