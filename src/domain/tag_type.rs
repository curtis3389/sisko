use std::fmt::Display;

/// Represents the type of a metadata tag.
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Display for TagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TagType::FLAC => "FLAC",
                TagType::ID3v1 => "ID3v1",
                TagType::ID3v2 => "ID3v2",
                TagType::Vorbis => "Vorbis",
            }
        )
    }
}
