use crate::domain::{File, Tag};

/// Represents an audio track.
pub struct Track {
    /// The title of the track.
    pub title: String,

    /// The artist of the track.
    pub artist: String,

    /// The length of the track.
    pub length: String,

    /// The file this track is from.
    pub file: File,

    /// The tags on the file.
    pub tags: Vec<Box<dyn Tag>>,
}
