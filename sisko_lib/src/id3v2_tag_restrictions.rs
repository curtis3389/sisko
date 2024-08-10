use crate::{is_bit_set, set_bit};

/// Represents the restrictions on a tag before encoding.
#[derive(Clone, Debug)]
pub struct ID3v2TagRestrictions {
    /// The restrictions on the tag size.
    pub tag_size: TagSizeRestriction,

    /// The restrictions on how text is encoding.
    pub text_encoding: TextEncodingRestriction,

    /// The restrictions on text field size.
    pub text_field_size: TextFieldSizeRestriction,

    /// The restrictions on image encoding.
    pub image_encoding: ImageEncodingRestriction,

    /// The restrictions on image size.
    pub image_size: ImageSizeRestriction,
}

impl ID3v2TagRestrictions {
    /// Parses the tag restrictions from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the restrictions from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// let r = ID3v2TagRestrictions::parse(0b0010_0110);
    ///
    /// assert_eq!(r.tag_size, TagSizeRestriction::Max128Frames1MB);
    /// assert_eq!(r.text_encoding, TextEncodingRestriction::Iso88591OrUtf8);
    /// assert_eq!(r.text_field_size, TextFieldSizeRestriction::NoRestrictions);
    /// assert_eq!(r.image_encoding, ImageEncodingRestriction::PngOrJpeg);
    /// assert_eq!(r.image_size, ImageSizeRestriction::Max64);
    /// ```
    pub fn parse(byte: u8) -> ID3v2TagRestrictions {
        ID3v2TagRestrictions {
            tag_size: TagSizeRestriction::from(byte),
            text_encoding: TextEncodingRestriction::from(byte),
            text_field_size: TextFieldSizeRestriction::from(byte),
            image_encoding: ImageEncodingRestriction::from(byte),
            image_size: ImageSizeRestriction::from(byte),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte = 0u8;
        match self.tag_size {
            TagSizeRestriction::Max128Frames1MB => {}
            TagSizeRestriction::Max64Frames128KB => set_bit(&mut byte, 6),
            TagSizeRestriction::Max32Frames40KB => set_bit(&mut byte, 7),
            TagSizeRestriction::Max32Frames4KB => {
                set_bit(&mut byte, 7);
                set_bit(&mut byte, 6);
            }
        }
        if self.text_encoding == TextEncodingRestriction::Iso88591OrUtf8 {
            set_bit(&mut byte, 5);
        }
        match self.text_field_size {
            TextFieldSizeRestriction::NoRestrictions => {}
            TextFieldSizeRestriction::Max1024 => set_bit(&mut byte, 3),
            TextFieldSizeRestriction::Max128 => set_bit(&mut byte, 4),
            TextFieldSizeRestriction::Max30 => {
                set_bit(&mut byte, 4);
                set_bit(&mut byte, 3);
            }
        }
        if self.image_encoding == ImageEncodingRestriction::PngOrJpeg {
            set_bit(&mut byte, 2);
        }
        match self.image_size {
            ImageSizeRestriction::NoRestrictions => {}
            ImageSizeRestriction::Max256 => set_bit(&mut byte, 0),
            ImageSizeRestriction::Max64 => set_bit(&mut byte, 1),
            ImageSizeRestriction::Exactly64 => {
                set_bit(&mut byte, 1);
                set_bit(&mut byte, 0);
            }
        }
        vec![byte]
    }
}

/// Represents possible tag size restrictions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TagSizeRestriction {
    /// No more than 128 frames and 1 MB total tag size.
    Max128Frames1MB,

    /// No more than 64 frames and 128 KB total tag size.
    Max64Frames128KB,

    /// No more than 32 frames and 40 KB total tag size.
    Max32Frames40KB,

    /// No more than 32 frames and 4 KB total tag size.
    Max32Frames4KB,
}

impl From<u8> for TagSizeRestriction {
    /// Converts the given byte to a TagSizeRestriction.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// assert_eq!(TagSizeRestriction::from(0b0000_0000), TagSizeRestriction::Max128Frames1MB);
    /// assert_eq!(TagSizeRestriction::from(0b0100_0000), TagSizeRestriction::Max64Frames128KB);
    /// assert_eq!(TagSizeRestriction::from(0b1000_0000), TagSizeRestriction::Max32Frames40KB);
    /// assert_eq!(TagSizeRestriction::from(0b1100_0000), TagSizeRestriction::Max32Frames4KB);
    /// ```
    fn from(byte: u8) -> Self {
        match (is_bit_set(byte, 7), is_bit_set(byte, 6)) {
            (false, false) => Self::Max128Frames1MB,
            (false, true) => Self::Max64Frames128KB,
            (true, false) => Self::Max32Frames40KB,
            (true, true) => Self::Max32Frames4KB,
        }
    }
}

/// Represents possible text encoding restrictions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextEncodingRestriction {
    /// No restrictions.
    NoRestrictions,

    /// Strings are only encoded with ISO-8859-1 or UTF-8.
    Iso88591OrUtf8,
}

impl From<u8> for TextEncodingRestriction {
    /// Converts the given byte to a TextEncodingRestriction.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// assert_eq!(TextEncodingRestriction::from(0b0000_0000), TextEncodingRestriction::NoRestrictions);
    /// assert_eq!(TextEncodingRestriction::from(0b0010_0000), TextEncodingRestriction::Iso88591OrUtf8);
    /// ```
    fn from(byte: u8) -> Self {
        match is_bit_set(byte, 5) {
            false => Self::NoRestrictions,
            true => Self::Iso88591OrUtf8,
        }
    }
}

/// Represents possible text field size restrictions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextFieldSizeRestriction {
    /// No restrictions.
    NoRestrictions,

    /// No string is longer than 1024 characters.
    Max1024,

    /// No string is longer than 128 characters.
    Max128,

    /// No string is longer than 30 characters.
    Max30,
}

impl From<u8> for TextFieldSizeRestriction {
    /// Converts the given byte to a TextFieldSizeRestriction.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// assert_eq!(TextFieldSizeRestriction::from(0b0000_0000), TextFieldSizeRestriction::NoRestrictions);
    /// assert_eq!(TextFieldSizeRestriction::from(0b0000_1000), TextFieldSizeRestriction::Max1024);
    /// assert_eq!(TextFieldSizeRestriction::from(0b0001_0000), TextFieldSizeRestriction::Max128);
    /// assert_eq!(TextFieldSizeRestriction::from(0b0001_1000), TextFieldSizeRestriction::Max30);
    /// ```
    fn from(byte: u8) -> Self {
        match (is_bit_set(byte, 4), is_bit_set(byte, 3)) {
            (false, false) => Self::NoRestrictions,
            (false, true) => Self::Max1024,
            (true, false) => Self::Max128,
            (true, true) => Self::Max30,
        }
    }
}

/// Represents possible image encoding restrictions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageEncodingRestriction {
    /// No restrictions.
    NoRestrictions,

    /// Images are encoded only with PNG or JPEG.
    PngOrJpeg,
}

impl From<u8> for ImageEncodingRestriction {
    /// Converts the given byte to a ImageEncodingRestriction.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// assert_eq!(ImageEncodingRestriction::from(0b0000_0000), ImageEncodingRestriction::NoRestrictions);
    /// assert_eq!(ImageEncodingRestriction::from(0b0000_0100), ImageEncodingRestriction::PngOrJpeg);
    /// ```
    fn from(byte: u8) -> Self {
        match is_bit_set(byte, 2) {
            false => Self::NoRestrictions,
            true => Self::PngOrJpeg,
        }
    }
}

/// Represents possible image size restrictions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageSizeRestriction {
    /// No restrictions.
    NoRestrictions,

    /// All images are 256x256 pixels or smaller.
    Max256,

    /// All images are 64x64 pixels or smaller.
    Max64,

    /// All images are exactly 64x64 pixels, unless required otherwise.
    Exactly64,
}

impl From<u8> for ImageSizeRestriction {
    /// Converts the given byte to a ImageSizeRestriction.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_tag_restrictions::*;
    /// assert_eq!(ImageSizeRestriction::from(0b0000_0000), ImageSizeRestriction::NoRestrictions);
    /// assert_eq!(ImageSizeRestriction::from(0b0000_0001), ImageSizeRestriction::Max256);
    /// assert_eq!(ImageSizeRestriction::from(0b0000_0010), ImageSizeRestriction::Max64);
    /// assert_eq!(ImageSizeRestriction::from(0b0000_0011), ImageSizeRestriction::Exactly64);
    /// ```
    fn from(byte: u8) -> Self {
        match (is_bit_set(byte, 1), is_bit_set(byte, 0)) {
            (false, false) => Self::NoRestrictions,
            (false, true) => Self::Max256,
            (true, false) => Self::Max64,
            (true, true) => Self::Exactly64,
        }
    }
}
