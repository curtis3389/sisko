use std::fmt::Display;

use crate::decode_utf16_strings;
use anyhow::{anyhow, Result};

/// Represents the possible encoding of text in an ID3v2 tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextEncoding {
    /// ISO-8859-1. Terminated with a single zero byte ($00).
    Iso88591,

    /// UTF-16 with BOM. Terminated with two zero bytes ($00 00).
    Utf16Bom,

    /// UTF-16BE without BOM. Terminated with two zero bytes ($00 00).
    Utf16Be,

    /// UTF-8. Terminated with a single zero byte ($00).
    Utf8,
}

impl TextEncoding {
    /// Decodes the strings from the given bytes using this text encoding.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to decode the strings from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::text_encoding::*;
    /// let bytes = [50, 48, 48, 56, 0, 78, 105, 110, 101, 32, 73, 110, 99, 104, 32, 78, 97, 105, 108, 115, 0];
    ///
    /// let strings = TextEncoding::Utf8.decode(&bytes)?;
    ///
    /// assert_eq!(strings.len(), 2);
    /// assert_eq!(strings[0], "2008");
    /// assert_eq!(strings[1], "Nine Inch Nails");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<String>> {
        Ok(match self {
            TextEncoding::Utf16Bom | TextEncoding::Utf16Be => decode_utf16_strings(bytes)?,
            TextEncoding::Iso88591 | TextEncoding::Utf8 => bytes
                .split(|&b| b == 0)
                .filter(|b| !b.is_empty())
                .map(|b| b.iter().map(|&c| c as char).collect())
                .collect(),
        })
    }

    /// Gets the index of the end of the next terminator for this encoding in the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to find the end of the next terminator in.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::text_encoding::*;
    /// let bytes = [50, 48, 48, 56, 0, 78, 105, 110, 101, 32, 73, 110, 99, 104, 32, 78, 97, 105, 108, 115, 0];
    ///
    /// let index = TextEncoding::Utf8.next_terminator(&bytes);
    ///
    /// assert_eq!(index, Some(4));
    /// ```
    pub fn next_terminator(&self, bytes: &[u8]) -> Option<usize> {
        match self {
            TextEncoding::Iso88591 | TextEncoding::Utf8 => {
                let all: Vec<(usize, &u8)> = bytes
                    .iter()
                    .enumerate()
                    .filter(|(_i, &b)| b == 0)
                    .take(1)
                    .collect();
                all.first().map(|(index, _)| *index)
            }
            TextEncoding::Utf16Bom | TextEncoding::Utf16Be => {
                let all: Vec<(usize, &u8)> = bytes
                    .iter()
                    .enumerate()
                    .filter(|(i, &b)| b == 0 && bytes[i + 1] == 0)
                    .take(1)
                    .collect();
                all.first().map(|(index, _)| *index)
            }
        }
    }

    /// Parses the text encoding from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the text encoding from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::text_encoding::*;
    /// assert_eq!(TextEncoding::parse(0)?, TextEncoding::Iso88591);
    /// assert_eq!(TextEncoding::parse(1)?, TextEncoding::Utf16Bom);
    /// assert_eq!(TextEncoding::parse(2)?, TextEncoding::Utf16Be);
    /// assert_eq!(TextEncoding::parse(3)?, TextEncoding::Utf8);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn parse(byte: u8) -> Result<TextEncoding> {
        match byte {
            0 => Ok(TextEncoding::Iso88591),
            1 => Ok(TextEncoding::Utf16Bom),
            2 => Ok(TextEncoding::Utf16Be),
            3 => Ok(TextEncoding::Utf8),
            _ => Err(anyhow!("Unknown text encoding: {}", byte)),
        }
    }

    /// Gets the width of the string terminator for this encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::text_encoding::*;
    /// assert_eq!(TextEncoding::Iso88591.terminator_width(), 1);
    /// assert_eq!(TextEncoding::Utf16Bom.terminator_width(), 2);
    /// assert_eq!(TextEncoding::Utf16Be.terminator_width(), 2);
    /// assert_eq!(TextEncoding::Utf8.terminator_width(), 1);
    /// ```
    pub fn terminator_width(&self) -> usize {
        match self {
            TextEncoding::Iso88591 => 1,
            TextEncoding::Utf16Bom => 2,
            TextEncoding::Utf16Be => 2,
            TextEncoding::Utf8 => 1,
        }
    }
}

impl Display for TextEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TextEncoding::Iso88591 => "ISO-8859-1",
                TextEncoding::Utf16Bom => "UTF-16 BOM",
                TextEncoding::Utf16Be => "UTF-16 BE",
                TextEncoding::Utf8 => "UTF-8",
            }
        )
    }
}
