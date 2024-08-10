use crate::{is_bit_set, set_bit};

/// Represents the status messages flags for a frame in an ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2FrameStatusMessages {
    /// Whether or not to discard this frame if it is unknown and the tag is being altered.
    pub preserve_on_alter_tag: bool,

    /// Whether or not to discard this frame if it is unknown and the file (excluding the tag) is altered.
    pub preserve_on_alter_file: bool,

    /// Whether or not the frame is intended to be read-only.
    /// If the contents of the frame is modified without knowing why it's read-only, this MUST be set false.
    pub is_read_only: bool,
}

impl ID3v2FrameStatusMessages {
    /// Parses the frame status messages flags from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the flags from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame_status_messages::*;
    /// let status_messages = ID3v2FrameStatusMessages::parse(0b0001_0000);
    ///
    /// assert_eq!(status_messages.preserve_on_alter_tag, true);
    /// assert_eq!(status_messages.preserve_on_alter_file, true);
    /// assert_eq!(status_messages.is_read_only, true);
    /// ```
    pub fn parse(byte: u8) -> ID3v2FrameStatusMessages {
        ID3v2FrameStatusMessages {
            preserve_on_alter_tag: !is_bit_set(byte, 6),
            preserve_on_alter_file: !is_bit_set(byte, 5),
            is_read_only: is_bit_set(byte, 4),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte = 0u8;
        if !self.preserve_on_alter_tag {
            set_bit(&mut byte, 6);
        }
        if !self.preserve_on_alter_file {
            set_bit(&mut byte, 5);
        }
        if self.is_read_only {
            set_bit(&mut byte, 4);
        }
        vec![byte]
    }
}
