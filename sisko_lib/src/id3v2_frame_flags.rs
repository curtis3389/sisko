use crate::id3v2_frame_format_description::ID3v2FrameFormatDescription;
use crate::id3v2_frame_status_messages::ID3v2FrameStatusMessages;

/// Represents the flags for a frame in an ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2FrameFlags {
    /// The status messages flags.
    pub status_messages: ID3v2FrameStatusMessages,

    /// The format description flags.
    pub format_description: ID3v2FrameFormatDescription,
}

impl ID3v2FrameFlags {
    /// Parses the frame flags from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the flags from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame_flags::*;
    /// let bytes = [0b0110_1010, 0b0101_0101];
    ///
    /// let flags = ID3v2FrameFlags::parse(&bytes);
    ///
    /// assert_eq!(flags.status_messages.preserve_on_alter_tag, false);
    /// assert_eq!(flags.status_messages.preserve_on_alter_file, false);
    /// assert_eq!(flags.status_messages.is_read_only, false);
    ///
    /// assert_eq!(flags.format_description.is_in_group, true);
    /// assert_eq!(flags.format_description.is_compressed, false);
    /// assert_eq!(flags.format_description.is_encrypted, true);
    /// assert_eq!(flags.format_description.is_unsynchronised, false);
    /// assert_eq!(
    ///     flags.format_description.has_data_length_indicator,
    ///     true
    /// );
    /// ```
    pub fn parse(bytes: &[u8]) -> ID3v2FrameFlags {
        let status_messages = ID3v2FrameStatusMessages::parse(bytes[0]);
        let format_description = ID3v2FrameFormatDescription::parse(bytes[1]);

        ID3v2FrameFlags {
            status_messages,
            format_description,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let status_bytes = self.status_messages.to_bytes();
        let format_bytes = self.format_description.to_bytes();

        let mut flag_bytes: Vec<u8> = vec![];
        flag_bytes.extend(status_bytes);
        flag_bytes.extend(format_bytes);
        flag_bytes
    }
}
