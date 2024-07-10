use crate::id3v2_header_flags::ID3v2HeaderFlags;
use crate::id3v2_version_number::ID3v2VersionNumber;
use crate::synch_safe_integer::SynchSafeInteger;
use anyhow::Result;

/// Represents the footer for an ID3v2 metadata tag.
#[derive(Clone, Debug)]
pub struct ID3v2Footer {
    /// The file identifier at the beginning of the tag.
    /// This is always "3DI".
    pub file_identifier: String,

    /// The version of the tag.
    pub version: ID3v2VersionNumber,

    /// The flags for the tag as a whole.
    pub flags: ID3v2HeaderFlags,

    /// The number of bytes in the tag excluding the header and footer.
    pub size: u32,
}

impl ID3v2Footer {
    /// Parses a new ID3v2Footer from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the footer from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_footer::*;
    /// let bytes = [b'3', b'D', b'I', b'\x03', b'\x00', b'\x00', b'\x00', b'\x00', b'\x21', b'\x79'];
    ///
    /// let footer = ID3v2Footer::parse(&bytes)?;
    ///
    /// assert_eq!(footer.file_identifier, "3DI");
    /// assert_eq!(footer.version.major_number, 3);
    /// assert_eq!(footer.version.revision_number, 0);
    /// assert_eq!(footer.flags.unsynchronisation, false);
    /// assert_eq!(footer.flags.has_extended_header, false);
    /// assert_eq!(footer.flags.is_experimental, false);
    /// assert_eq!(footer.flags.has_footer, false);
    /// assert_eq!(footer.size, 4345);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn parse(bytes: &[u8; 10]) -> Result<ID3v2Footer> {
        let file_identifier = String::from_utf8(bytes[0..3].to_vec())?;
        let version = ID3v2VersionNumber::new(bytes[3], bytes[4]);
        let flags = ID3v2HeaderFlags::parse(bytes[5]);
        let size = u32::from(SynchSafeInteger::new(&bytes[6..10]));

        Ok(ID3v2Footer {
            file_identifier,
            version,
            flags,
            size,
        })
    }
}
