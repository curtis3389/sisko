use crate::id3v2_header_flags::ID3v2HeaderFlags;
use crate::id3v2_version_number::ID3v2VersionNumber;
use crate::synch_safe_integer::SynchSafeInteger;
use anyhow::Result;

/// Represents the header for an ID3v2 metadata tag.
#[derive(Clone, Debug)]
pub struct ID3v2Header {
    /// The file identifier at the beginning of the tag.
    /// This is always "ID3".
    pub file_identifier: String,

    /// The version of the tag.
    pub version: ID3v2VersionNumber,

    /// The flags for the tag as a whole.
    pub flags: ID3v2HeaderFlags,

    /// The number of bytes in the tag excluding the header and footer.
    pub size: u32,
}

impl ID3v2Header {
    /// Parses an ID3v2 header from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the header from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_header::*;
    /// let bytes = [b'I', b'D', b'3', b'\x03', b'\x00', b'\x00', b'\x00', b'\x00', b'\x21', b'\x79'];
    ///
    /// let header = ID3v2Header::parse(&bytes)?;
    ///
    /// assert_eq!(header.file_identifier, "ID3");
    /// assert_eq!(header.version.major_number, 3);
    /// assert_eq!(header.version.revision_number, 0);
    /// assert_eq!(header.flags.unsynchronisation, false);
    /// assert_eq!(header.flags.has_extended_header, false);
    /// assert_eq!(header.flags.is_experimental, false);
    /// assert_eq!(header.flags.has_footer, false);
    /// assert_eq!(header.size, 4345);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn parse(bytes: &[u8; 10]) -> Result<ID3v2Header> {
        let file_identifier = String::from_utf8(bytes[0..3].to_vec())?;
        let version = ID3v2VersionNumber::new(bytes[3], bytes[4]);
        let flags = ID3v2HeaderFlags::parse(bytes[5]);
        let size = u32::from(SynchSafeInteger::new(&bytes[6..10]));

        Ok(ID3v2Header {
            file_identifier,
            version,
            flags,
            size,
        })
    }
}
