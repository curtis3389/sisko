use crate::infrastructure::Value;
use std::fmt::Display;

/// Represents the type of a metadata tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TagType {
    /// A FLAC tag.
    FLAC,

    /// An ID3v1 tag.
    ID3v1,

    /// An ID3v2 tag.
    ID3v2,

    /// A Vorbis tag.
    Vorbis,
}

impl TagType {
    pub fn as_str(&self) -> &str {
        match &self {
            TagType::FLAC => "FLAC",
            TagType::ID3v1 => "ID3v1",
            TagType::ID3v2 => "ID3v2",
            TagType::Vorbis => "Vorbis",
        }
    }
}

impl Display for TagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for TagType {
    fn from(s: &str) -> Self {
        match s {
            "FLAC" => TagType::FLAC,
            "ID3v1" => TagType::ID3v1,
            "ID3v2" => TagType::ID3v2,
            "Vorbis" => TagType::Vorbis,
            _ => panic!("Unknown tag type: {}!", s),
        }
    }
}

impl Value for TagType {}
