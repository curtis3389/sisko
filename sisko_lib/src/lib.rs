//! This library contains the types the sisko CLI tool uses to work with audio files.

use anyhow::{anyhow, Result};
use encoding_rs::UTF_16BE;

pub mod id3v2_extended_flag_data;
pub mod id3v2_extended_flags;
pub mod id3v2_extended_header;
pub mod id3v2_footer;
pub mod id3v2_frame;
pub mod id3v2_frame_fields;
pub mod id3v2_frame_flags;
pub mod id3v2_frame_format_description;
pub mod id3v2_frame_header;
pub mod id3v2_frame_status_messages;
pub mod id3v2_header;
pub mod id3v2_header_flags;
pub mod id3v2_tag;
pub mod id3v2_tag_restrictions;
pub mod id3v2_version_number;
pub mod picture_type;
pub mod synch_safe_integer;
pub mod text_encoding;

/// Returns whether or not the given byte has the given bit set.
/// Bits are numbers as follows: 0b7654_3210.
///
/// # Arguments
///
/// * `byte` - The byte to check.
/// * `bit_number` - The number of the bit to check if is set.
///
/// # Examples
///
/// ```
/// # use sisko_lib::*;
/// let byte = 0b0101_0101;
///
/// assert!(!is_bit_set(byte, 7));
/// assert!(is_bit_set(byte, 6));
/// assert!(!is_bit_set(byte, 5));
/// assert!(is_bit_set(byte, 4));
///
/// assert!(!is_bit_set(byte, 3));
/// assert!(is_bit_set(byte, 2));
/// assert!(!is_bit_set(byte, 1));
/// assert!(is_bit_set(byte, 0));
/// ```
pub fn is_bit_set(byte: u8, bit_number: usize) -> bool {
    let mask = 0b0000_0001 << bit_number;
    (byte & mask) != 0
}

/// Decodes the UTF-16 strings from the given array of bytes.
/// Strings are terminated by nulls.
/// Byte order will be deduced if a BOM is present; otherwise, big-endian is assumed.
///
/// # Arguments
///
/// * `bytes` - The bytes to decode the strings from.
///
/// # Examples
///
/// ```
/// # use sisko_lib::*;
/// let bytes = [b'\xff', b'\xfe', b'\x32', 0, b'\x30', 0, b'\x30', 0, b'\x38', 0, 0, 0];
///
/// let strings = decode_utf16_strings(&bytes).unwrap();
///
/// assert_eq!(strings.len(), 1);
/// assert_eq!(&strings[0], "2008");
/// ```
pub fn decode_utf16_strings(bytes: &[u8]) -> Result<Vec<String>> {
    let (utf16, _, _) = UTF_16BE.decode(bytes);
    let mut strings: Vec<String> = utf16
        .split(|c| c == '\x00')
        .map(|chars| String::from(chars))
        .map(|s| s.replace("\u{feff}", ""))
        .collect();
    if strings
        .last()
        .ok_or(anyhow!("No strings parsed for UTF-16 encoding!"))?
        == ""
    {
        strings.pop();
    }
    Ok(strings)
}
