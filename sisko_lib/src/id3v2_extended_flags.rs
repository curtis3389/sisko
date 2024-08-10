use crate::{is_bit_set, set_bit};

/// Represents the flags in and ID3v2 extended header.
#[derive(Clone, Debug)]
pub struct ID3v2ExtendedFlags {
    /// Whether or not the tag is an update of a previous tag in the file/stream.
    pub is_update: bool,

    /// Whether or not CRC-32 (ISO-3309) data is included in the extended header.
    pub has_crc: bool,

    /// Whether or not the tag has restrictions.
    pub has_restrictions: bool,

    /// Whether or not bit 3 is set.
    pub unknown_flag_3: bool,

    /// Whether or not bit 2 is set.
    pub unknown_flag_2: bool,

    /// Whether or not bit 1 is set.
    pub unknown_flag_1: bool,

    /// Whether or not bit 0 is set.
    pub unknown_flag_0: bool,
}

impl ID3v2ExtendedFlags {
    /// Parses the extended header flags from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the flags from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_extended_flags::*;
    /// let flags = ID3v2ExtendedFlags::parse(0b0011_0000);
    ///
    /// assert_eq!(flags.is_update, false);
    /// assert_eq!(flags.has_crc, true);
    /// assert_eq!(flags.has_restrictions, true);
    ///
    /// assert_eq!(flags.unknown_flag_3, false);
    /// assert_eq!(flags.unknown_flag_2, false);
    /// assert_eq!(flags.unknown_flag_1, false);
    /// assert_eq!(flags.unknown_flag_0, false);
    /// ```
    pub fn parse(byte: u8) -> ID3v2ExtendedFlags {
        ID3v2ExtendedFlags {
            is_update: is_bit_set(byte, 6),
            has_crc: is_bit_set(byte, 5),
            has_restrictions: is_bit_set(byte, 4),
            unknown_flag_3: is_bit_set(byte, 3),
            unknown_flag_2: is_bit_set(byte, 2),
            unknown_flag_1: is_bit_set(byte, 1),
            unknown_flag_0: is_bit_set(byte, 0),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte = 0u8;
        if self.is_update {
            set_bit(&mut byte, 6);
        }
        if self.has_crc {
            set_bit(&mut byte, 5);
        }
        if self.has_restrictions {
            set_bit(&mut byte, 4);
        }
        if self.unknown_flag_3 {
            set_bit(&mut byte, 3);
        }
        if self.unknown_flag_2 {
            set_bit(&mut byte, 2);
        }
        if self.unknown_flag_1 {
            set_bit(&mut byte, 1);
        }
        if self.unknown_flag_0 {
            set_bit(&mut byte, 0);
        }
        vec![byte]
    }
}
