use crate::id3v2_frame_flags::ID3v2FrameFlags;
use crate::id3v2_version_number::ID3v2VersionNumber;
use crate::synch_safe_integer::SynchSafeInteger;

/// Represents the header of a frame in an ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2FrameHeader {
    /// The ID of the frame.
    pub frame_id: String,

    /// The number of bytes of the final frame after encryption, compression, and
    /// unsynchronisation.
    pub size: u32,

    /// The flags for the frame.
    pub flags: ID3v2FrameFlags,
}

impl ID3v2FrameHeader {
    /// Parses a frame header from the given bytes.
    ///
    /// # Arguments
    /// `bytes` - The bytes to parse the header from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame_header::*;
    /// # use sisko_lib::id3v2_version_number::*;
    /// let bytes = [b'\x54', b'\x52', b'\x43', b'\x4b', b'\x00', b'\x00', b'\x00', b'\x06', b'\x00', b'\x00'];
    /// let version = ID3v2VersionNumber { major_number: 4, revision_number: 0 };
    ///
    /// let header = ID3v2FrameHeader::parse(&bytes, &version);
    ///
    /// assert_eq!(header.frame_id, "TRCK");
    /// assert_eq!(header.size, 6);
    ///
    /// assert_eq!(header.flags.status_messages.preserve_on_alter_tag, false);
    /// assert_eq!(header.flags.status_messages.preserve_on_alter_file, false);
    /// assert_eq!(header.flags.status_messages.is_read_only, false);
    ///
    /// assert_eq!(header.flags.format_description.is_in_group, false);
    /// assert_eq!(header.flags.format_description.is_compressed, false);
    /// assert_eq!(header.flags.format_description.is_encrypted, false);
    /// assert_eq!(header.flags.format_description.is_unsynchronised, false);
    /// assert_eq!(
    ///     header.flags.format_description.has_data_length_indicator,
    ///     false
    /// );
    /// ```
    pub fn parse(bytes: &[u8], version: &ID3v2VersionNumber) -> ID3v2FrameHeader {
        let frame_id = String::from_utf8(bytes[0..4].iter().map(|b| *b).collect()).unwrap();
        let size = match version.major_number {
            4 => u32::from(SynchSafeInteger::new(&bytes[4..8])),
            _ => u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        };
        let flags = ID3v2FrameFlags::parse(&bytes[8..10]);

        ID3v2FrameHeader {
            frame_id,
            size,
            flags,
        }
    }
}
