#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TagType {
    FLAC,
    ID3v1,
    ID3v2,
    Vorbis,
}
