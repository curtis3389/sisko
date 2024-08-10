use crate::id3v2_extended_flags::ID3v2ExtendedFlags;
use crate::id3v2_tag_restrictions::ID3v2TagRestrictions;
use crate::synch_safe_integer::SynchSafeInteger;
use std::fmt::Debug;

/// Represents the data for a set flag in an extended header of an ID3v2 tag.
#[derive(Clone, Debug)]
pub enum ID3v2ExtendedFlagData {
    /// The data for the "tag is an update" flag, which has no data.
    TagIsUpdateData {
        /// The length of the flag's data, in bytes.
        length: u8,
    },
    /// The data for the "CRC data present" flag.
    CrcPresentData {
        /// The length of the flag's data, in bytes.
        length: u8,

        /// The CRC-32 checksum for the tag excluding the header, extended header, and footer.
        crc: u32,
    },
    /// The data for the tag restrictions flag.
    TagRestrictionsData {
        /// The length of the flag's data, in bytes.
        length: u8,

        /// The restrictions on the tag before it was encoded.
        restrictions: ID3v2TagRestrictions,
    },
    /// The data for an unknown extended header flag.
    UnknownData {
        /// The length of the flag's data, in bytes.
        length: u8,

        /// The bytes of the flag's data.
        bytes: Vec<u8>,
    },
}

impl ID3v2ExtendedFlagData {
    /// Parses all of extended flag data for the given extended flags from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the data from.
    /// * `extended_flags` - The flags to parse the data for.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_extended_flag_data::*;
    /// # use sisko_lib::id3v2_extended_flags::*;
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// let bytes = [0, 5, 0, 0, 0b0000_0100, 0b0001_1110, 0b0010_1100, 1, 0b0110_0111];
    /// let flags = ID3v2ExtendedFlags {
    ///     is_update: true,
    ///     has_crc: true,
    ///     has_restrictions: true,
    ///     unknown_flag_3: false,
    ///     unknown_flag_2: false,
    ///     unknown_flag_1: false,
    ///     unknown_flag_0: false,
    /// };
    ///
    /// let data = ID3v2ExtendedFlagData::parse_all(&bytes, &flags);
    ///
    /// if let ID3v2ExtendedFlagData::TagIsUpdateData { length } = &data[0] {
    ///     assert_eq!(*length, 0);
    /// } else { panic!(); }
    ///
    /// if let ID3v2ExtendedFlagData::CrcPresentData { length, crc } = &data[1] {
    ///     assert_eq!(*length, 5);
    ///     assert_eq!(*crc, 69420);
    /// } else { panic!(); }
    ///
    /// if let ID3v2ExtendedFlagData::TagRestrictionsData { length, restrictions } = &data[2] {
    ///     assert_eq!(*length, 1);
    ///     assert_eq!(restrictions.tag_size, TagSizeRestriction::Max64Frames128KB);
    ///     assert_eq!(restrictions.text_encoding, TextEncodingRestriction::Iso88591OrUtf8);
    ///     assert_eq!(restrictions.text_field_size, TextFieldSizeRestriction::NoRestrictions);
    ///     assert_eq!(restrictions.image_encoding, ImageEncodingRestriction::PngOrJpeg);
    ///     assert_eq!(restrictions.image_size, ImageSizeRestriction::Exactly64);
    /// } else { panic!(); }
    /// ```
    pub fn parse_all(
        bytes: &[u8],
        extended_flags: &ID3v2ExtendedFlags,
    ) -> Vec<ID3v2ExtendedFlagData> {
        let mut data: Vec<ID3v2ExtendedFlagData> = Vec::new();
        let mut index = 0;

        if extended_flags.is_update {
            data.push(ID3v2ExtendedFlagData::TagIsUpdateData {
                length: bytes[index],
            });
            index += 1;
        }

        if extended_flags.has_crc {
            let bytes = &bytes[index..(index + 6)];
            data.push(ID3v2ExtendedFlagData::CrcPresentData {
                length: bytes[0],
                crc: u32::from(SynchSafeInteger::new(&bytes[1..6])),
            });
            index += 6;
        }

        if extended_flags.has_restrictions {
            let bytes = &bytes[index..(index + 2)];
            data.push(ID3v2ExtendedFlagData::TagRestrictionsData {
                length: bytes[0],
                restrictions: ID3v2TagRestrictions::parse(bytes[1]),
            });
            index += 2;
        }

        let unknown = [
            extended_flags.unknown_flag_3,
            extended_flags.unknown_flag_2,
            extended_flags.unknown_flag_1,
            extended_flags.unknown_flag_0,
        ];
        for _unknown_flag in unknown.iter().filter(|&u| *u) {
            let bytes = &bytes[index..];
            let length = bytes[0];
            let bytes = bytes[1..(length as usize + 1)].to_vec();
            let unknown_data = ID3v2ExtendedFlagData::UnknownData { length, bytes };
            index += (unknown_data.length() as usize) + 1;
            data.push(unknown_data);
        }

        data
    }

    /// Returns the number of bytes the flag's data is.
    fn length(&self) -> u8 {
        match &self {
            ID3v2ExtendedFlagData::TagIsUpdateData { length } => *length,
            ID3v2ExtendedFlagData::CrcPresentData { length, crc: _ } => *length,
            ID3v2ExtendedFlagData::TagRestrictionsData {
                length,
                restrictions: _,
            } => *length,
            ID3v2ExtendedFlagData::UnknownData { length, bytes: _ } => *length,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ID3v2ExtendedFlagData::TagIsUpdateData { length } => vec![*length],
            ID3v2ExtendedFlagData::CrcPresentData { length, crc } => {
                let mut b: Vec<u8> = vec![*length];
                b.extend(SynchSafeInteger::from_5byte(*crc).bytes);
                b
            }
            ID3v2ExtendedFlagData::TagRestrictionsData {
                length,
                restrictions,
            } => {
                let mut b: Vec<u8> = vec![*length];
                b.extend(restrictions.to_bytes());
                b
            }
            ID3v2ExtendedFlagData::UnknownData { length, bytes } => {
                let mut b: Vec<u8> = vec![*length];
                b.extend(bytes);
                b
            }
        }
    }
}
