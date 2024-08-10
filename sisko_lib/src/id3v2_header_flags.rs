use crate::{is_bit_set, set_bit};

/// Represents the flags in an ID3v2 header.
#[derive(Clone, Debug)]
pub struct ID3v2HeaderFlags {
    /// Whether or not unsynchronisation is applied on all frames.
    pub unsynchronisation: bool,

    /// Whether or not the header is followed by an extended header.
    pub has_extended_header: bool,

    /// Whether or not the tag is in an experimental stage.
    pub is_experimental: bool,

    /// Whether or not a footer is present.
    pub has_footer: bool,
}

impl ID3v2HeaderFlags {
    /// Parses the flags for an ID3v2 header from the given flags byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The flags byte for an ID3v2 header.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_header_flags::*;
    /// let byte = 0b1101_0000;
    ///
    /// let flags = ID3v2HeaderFlags::parse(byte);
    ///
    /// assert_eq!(flags.unsynchronisation, true);
    /// assert_eq!(flags.has_extended_header, true);
    /// assert_eq!(flags.is_experimental, false);
    /// assert_eq!(flags.has_footer, true);
    /// ```
    pub fn parse(byte: u8) -> ID3v2HeaderFlags {
        let unsynchronisation = is_bit_set(byte, 7);
        let has_extended_header = is_bit_set(byte, 6);
        let is_experimental = is_bit_set(byte, 5);
        let has_footer = is_bit_set(byte, 4);

        ID3v2HeaderFlags {
            unsynchronisation,
            has_extended_header,
            is_experimental,
            has_footer,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte = 0u8;
        if self.unsynchronisation {
            set_bit(&mut byte, 7);
        }
        if self.has_extended_header {
            set_bit(&mut byte, 6);
        }
        if self.is_experimental {
            set_bit(&mut byte, 5);
        }
        if self.has_footer {
            set_bit(&mut byte, 4);
        }
        vec![byte]
    }
}
