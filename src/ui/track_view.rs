use crate::domain::Track;
use crate::ui::TrackColumn;
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

/// Represents an audio track.
#[derive(Clone)]
pub struct TrackView {
    pub title: String,
    pub artist: String,
    pub length: String,
    pub track: Arc<Mutex<Track>>,
}

impl From<&Arc<Mutex<Track>>> for TrackView {
    fn from(track: &Arc<Mutex<Track>>) -> Self {
        let mutex = track.clone();
        let track = track.lock().unwrap();
        Self {
            title: track.title().unwrap_or("<no title>".to_string()),
            artist: track.artist().unwrap_or("<no artist>".to_string()),
            length: track.length().unwrap_or("?:??".to_string()),
            track: mutex,
        }
    }
}

impl TableViewItem<TrackColumn> for TrackView {
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
