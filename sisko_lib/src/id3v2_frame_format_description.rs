use crate::is_bit_set;

/// Represents the format description flags for a frame in and ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2FrameFormatDescription {
    /// Whether or not the frame belongs to a group with other frames.
    pub is_in_group: bool,

    /// Whether or not the frame is compressed.
    pub is_compressed: bool,

    /// Whether or not the frame is encrypted.
    pub is_encrypted: bool,

    /// Whether or not the frame has been unsynchronised.
    pub is_unsynchronised: bool,

    /// Whether or not a data length indicator has been added to the frame.
    pub has_data_length_indicator: bool,
}

impl ID3v2FrameFormatDescription {
    /// Parses the frame format description flags from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the flags from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame_format_description::*;
    /// let format_description = ID3v2FrameFormatDescription::parse(0b0010_1010);
    ///
    /// assert_eq!(format_description.is_in_group, false);
    /// assert_eq!(format_description.is_compressed, true);
    /// assert_eq!(format_description.is_encrypted, false);
    /// assert_eq!(format_description.is_unsynchronised, true);
    /// assert_eq!(
    ///     format_description.has_data_length_indicator,
    ///     false
    /// );
    /// ```
    pub fn parse(byte: u8) -> ID3v2FrameFormatDescription {
        ID3v2FrameFormatDescription {
            is_in_group: is_bit_set(byte, 6),
            is_compressed: is_bit_set(byte, 3),
            is_encrypted: is_bit_set(byte, 2),
            is_unsynchronised: is_bit_set(byte, 1),
            has_data_length_indicator: is_bit_set(byte, 0),
        }
    }
}
