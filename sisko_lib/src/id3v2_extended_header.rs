use crate::id3v2_extended_flag_data::ID3v2ExtendedFlagData;
use crate::id3v2_extended_flags::ID3v2ExtendedFlags;
use crate::synch_safe_integer::SynchSafeInteger;

/// Represents the extended header for and ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2ExtendedHeader {
    /// The size of the whole extended header.
    pub size: u32,

    /// The number of bytes the extended flags take up.
    pub number_of_flag_bytes: u8,

    /// The extended header's flags.
    pub extended_flags: ID3v2ExtendedFlags,

    /// The data for the extended header's flags.
    pub extended_flag_data: Vec<ID3v2ExtendedFlagData>,
}

impl ID3v2ExtendedHeader {
    /// Parses an ID3v2 extended header from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse the extended header from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_extended_header::*;
    /// let bytes = [0, 0, 0, 6, 0, 0];
    ///
    /// let extended_header = ID3v2ExtendedHeader::parse(&bytes);
    ///
    /// assert_eq!(extended_header.size, 6);
    /// assert_eq!(extended_header.number_of_flag_bytes, 0);
    ///
    /// assert_eq!(extended_header.extended_flags.is_update, false);
    /// assert_eq!(extended_header.extended_flags.has_crc, false);
    /// assert_eq!(extended_header.extended_flags.has_restrictions, false);
    ///
    /// assert_eq!(extended_header.extended_flags.unknown_flag_3, false);
    /// assert_eq!(extended_header.extended_flags.unknown_flag_2, false);
    /// assert_eq!(extended_header.extended_flags.unknown_flag_1, false);
    /// assert_eq!(extended_header.extended_flags.unknown_flag_0, false);
    /// assert_eq!(extended_header.extended_flag_data.len(), 0);
    /// ```
    pub fn parse(bytes: &[u8]) -> ID3v2ExtendedHeader {
        let size = u32::from(SynchSafeInteger::new(&bytes[0..4]));
        let number_of_flag_bytes = bytes[4];
        let extended_flags = ID3v2ExtendedFlags::parse(bytes[5]);
        let extended_flag_data = ID3v2ExtendedFlagData::parse_all(&bytes[6..], &extended_flags);

        ID3v2ExtendedHeader {
            size,
            number_of_flag_bytes,
            extended_flags,
            extended_flag_data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let size_bytes = SynchSafeInteger::from(self.size).bytes;
        let num_bytes = vec![self.number_of_flag_bytes];
        let flag_bytes = self.extended_flags.to_bytes();
        let flag_data_bytes: Vec<u8> = self
            .extended_flag_data
            .iter()
            .flat_map(|d| d.to_bytes())
            .collect();

        let mut header_bytes = vec![];
        header_bytes.extend(size_bytes);
        header_bytes.extend(num_bytes);
        header_bytes.extend(flag_bytes);
        header_bytes.extend(flag_data_bytes);
        header_bytes
    }
}
