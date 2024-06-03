/// Represents the possible columns in a track table.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TrackColumn {
    /// The track title column.
    Title,
    /// The track artist column.
    Artist,
    /// The track length column.
    Length,
}

impl TrackColumn {
    /// Returns the display string for this track column.
    pub fn as_str(&self) -> &str {
        match *self {
            TrackColumn::Title => "Title",
            TrackColumn::Artist => "Artist",
            TrackColumn::Length => "Length",
        }
    }
}
