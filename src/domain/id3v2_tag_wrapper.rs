use crate::domain::{Tag, TagField};
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;

/// Represents a wrapper for the sisko_lib ID3v2Tag.
#[derive(Clone, Debug)]
pub struct ID3v2TagWrapper {
    /// The actual ID3v2 tag.
    tag: ID3v2Tag,
}

impl ID3v2TagWrapper {
    /// Returns a new wrapped ID3v2Tag for the given tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to wrap.
    pub fn new(tag: ID3v2Tag) -> Self {
        ID3v2TagWrapper { tag }
    }

    /// Gets the value of the text field with the given ID.
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

impl Tag for ID3v2TagWrapper {
    fn artist(&self) -> Option<String> {
        self.get_text_field("TPE1")
    }

    fn tag_name(&self) -> String {
        "ID3v2".to_string()
    }

    fn title(&self) -> Option<String> {
        self.get_text_field("TIT2")
    }

    fn fields(&self) -> Vec<TagField> {
        self.tag
            .frames
            .iter()
            .map(|frame| TagField {
                field_name: frame.header.frame_id.clone(),
                field_value: match &frame.fields {
                    ID3v2FrameFields::TextFields { encoding: _, text } => text[0].clone(),
                    _ => String::new(),
                },
                new_field_value: None,
            })
            .collect()
    }
}
