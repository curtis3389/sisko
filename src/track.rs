use crate::file::File;
use crate::track_column::TrackColumn;
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;

/// Represents an audio track.
#[derive(Clone, Debug)]
pub struct Track {
    /// The title of the track.
    pub title: String,
    /// The artist of the track.
    pub artist: String,
    /// The length of the track.
    pub length: String,
    /// The file this track is from.
    pub file: File,
    // The audio tag for this track.
    //pub tag: Rc<dyn AudioTag>,
}

impl TableViewItem<TrackColumn> for Track {
    /// Returns the value of the given column for this Track.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: TrackColumn) -> String {
        match column {
            TrackColumn::Title => self.title.to_string(),
            TrackColumn::Artist => self.artist.to_string(),
            TrackColumn::Length => self.length.to_string(),
        }
    }

    /// Compares the value of the given column to another Track.
    ///
    /// # Arguments
    ///
    /// * `other` - The other Track to compare to.
    /// * `column` - The column to compare between the Tracks.
    fn cmp(&self, other: &Self, column: TrackColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            TrackColumn::Title => self.title.cmp(&other.title),
            TrackColumn::Artist => self.artist.cmp(&other.artist),
            TrackColumn::Length => self.length.cmp(&other.length),
        }
    }
}
