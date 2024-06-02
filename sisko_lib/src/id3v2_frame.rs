use crate::id3v2_frame_fields::ID3v2FrameFields;
use crate::id3v2_frame_header::ID3v2FrameHeader;
use crate::id3v2_version_number::ID3v2VersionNumber;

/// Represents a frame in an ID3v2 tag.
/// This is what is common referred to as a "tag" (e.g. the artist tag).
#[derive(Clone, Debug)]
pub struct ID3v2Frame {
    /// The header for the frame.
    pub header: ID3v2FrameHeader,

    /// The fields of the frame.
    pub fields: ID3v2FrameFields,
}

impl ID3v2Frame {
    /// Parses an ID3v2 frame for the given ID3v2 version from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the frame from.
    /// * `version` - The version of ID3v2 to parse the frame for.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame::*;
    /// # use sisko_lib::id3v2_frame_fields::*;
    /// # use sisko_lib::id3v2_version_number::*;
    /// # use sisko_lib::text_encoding::*;
    /// let bytes = [b'\x54', b'\x50', b'\x4f', b'\x53', b'\x00', b'\x00', b'\x00', b'\x05', b'\x00', b'\x00', b'\x00', b'\x31', b'\x2f', b'\x32', b'\x00'];
    /// let version = ID3v2VersionNumber { major_number: 4, revision_number: 0 };
    ///
    /// let frame = ID3v2Frame::parse(&bytes, &version);
    ///
    /// assert_eq!(frame.header.frame_id, "TPOS");
    /// assert_eq!(frame.header.size, 5);
    ///
    /// if let ID3v2FrameFields::TextFields { encoding, text } = frame.fields {
    ///     assert_eq!(encoding, TextEncoding::Iso88591);
    ///     assert_eq!(text, vec!("1/2"));
    /// } else {
    ///     panic!();
    /// }
    /// ```
    pub fn parse(bytes: &[u8], version: &ID3v2VersionNumber) -> ID3v2Frame {
        let header = ID3v2FrameHeader::parse(&bytes[..10], version);
        let fields = ID3v2FrameFields::parse(&header, &bytes[10..(header.size as usize + 10)]);

        ID3v2Frame { header, fields }
    }

    /// Parses all of the ID3v2 frames for the given ID3v2 version from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the frames from.
    /// * `version` - The version of ID3v2 to parse the frames for.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame::*;
    /// # use sisko_lib::id3v2_frame_fields::*;
    /// # use sisko_lib::id3v2_version_number::*;
    /// # use sisko_lib::text_encoding::*;
    /// let bytes = [b'\x54', b'\x50', b'\x4f', b'\x53', b'\x00', b'\x00', b'\x00', b'\x05', b'\x00', b'\x00', b'\x00', b'\x31', b'\x2f', b'\x32', b'\x00'];
    /// let version = ID3v2VersionNumber { major_number: 4, revision_number: 0 };
    ///
    /// let mut frames = ID3v2Frame::parse_all(&bytes, &version);
    ///
    /// let frame = frames.remove(0);
    /// assert_eq!(frame.header.frame_id, "TPOS");
    /// assert_eq!(frame.header.size, 5);
    ///
    /// if let ID3v2FrameFields::TextFields { encoding, text } = frame.fields {
    ///     assert_eq!(encoding, TextEncoding::Iso88591);
    ///     assert_eq!(text, vec!("1/2"));
    /// } else {
    ///     panic!();
    /// }
    /// ```
    pub fn parse_all(bytes: &[u8], version: &ID3v2VersionNumber) -> Vec<ID3v2Frame> {
        let mut frames: Vec<ID3v2Frame> = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            let frame = ID3v2Frame::parse(&bytes[index..], version);

            if frame.header.size == 0 {
                break;
            }

            index += 10 + (frame.header.size as usize);
            frames.push(frame);
        }
        frames
    }
}
