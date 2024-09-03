use super::AudioFileColumn;
use crate::domain::{Album, Track};
use crate::infrastructure::DurationExtensions;
use anyhow::Result;
use cursive_table_view::TableViewItem;
use log::error;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct AlbumView {
    pub id: String,
    pub album_id: String,
    pub album: Album,
    pub track_id: Option<String>,
}

impl AlbumView {
    pub fn for_album(album: &Album) -> Result<Vec<AlbumView>> {
        let mut views = vec![];
        views.push(AlbumView::from(album));
        let mut tracks: Vec<AlbumView> = album
            .tracks
            .iter()
            .map(|track| AlbumView::for_track(album, album.id.clone(), track))
            .collect();
        views.append(&mut tracks);
        Ok(views)
    }

    pub fn for_track(album: &Album, album_id: String, track: &Track) -> AlbumView {
        Self {
            id: track.id.clone(),
            album_id,
            album: album.clone(),
            track_id: Some(track.id.clone()),
        }
    }

    pub fn artist(&self) -> Result<String> {
        match &self.track_id {
            None => Ok(self.album.artist.clone()),
            Some(track_id) => Ok(self.album.track(track_id)?.artist.clone()),
        }
    }

    pub fn disc_number(&self) -> Result<i32> {
        match &self.track_id {
            None => Ok(0),
            Some(track_id) => Ok(self.album.track(track_id)?.disc_number),
        }
    }

    pub fn length(&self) -> Result<String> {
        match &self.track_id {
            None => Ok(self.album.length.to_pretty_string()),
            Some(track_id) => Ok(self.album.track(track_id)?.length.to_pretty_string()),
        }
    }

    pub fn number(&self) -> Result<i32> {
        match &self.track_id {
            None => Ok(0),
            Some(track_id) => Ok(self.album.track(track_id)?.number),
        }
    }

    pub fn title(&self) -> Result<String> {
        match &self.track_id {
            None => {
                let icon = match (self.album.is_all_matched(), self.album.has_changes()) {
                    (true, false) => "⦿",
                    (false, false) => "⦾",
                    (true, true) => "⦿⃰",
                    (false, true) => "⦾⃰",
                };
                Ok(format!("{} {}", icon, self.album.title.clone()))
            }
            Some(track_id) => {
                let track = self.album.track(track_id)?;
                let icon = match (track.has_match(), track.has_changes()) {
                    (true, false) => "✔",
                    (false, false) => " ",
                    (true, true) => "█", // TODO: use match rating to use other block characters: ▁, ▂, ▃, ▄, ▅, ▆, ▇
                    (false, true) => " ",
                };
                Ok(format!(
                    "  {} {}-{} {}",
                    icon, track.disc_number, track.number, track.title
                ))
            }
        }
    }
}

impl From<&Album> for AlbumView {
    fn from(album: &Album) -> Self {
        Self {
            id: album.id.clone(),
            album_id: album.id.clone(),
            album: album.clone(),
            track_id: None,
        }
    }
}

impl TableViewItem<AudioFileColumn> for AlbumView {
    /// Returns the value of the given column for this AudioFile.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: AudioFileColumn) -> String {
        match column {
            AudioFileColumn::Title => self.title().unwrap_or_else(|e| e.to_string()), // TODO: log error as well
            AudioFileColumn::Artist => self.artist().unwrap_or_else(|e| e.to_string()),
            AudioFileColumn::Length => self.length().unwrap_or_else(|e| e.to_string()),
        }
    }

    /// Compares the value of the given column to another AudioFile.
    ///
    /// # Arguments
    ///
    /// * `other` - The other AudioFile to compare to.
    /// * `column` - The column to compare between the AudioFiles.
    fn cmp(&self, other: &Self, column: AudioFileColumn) -> Ordering
    where
        Self: Sized,
    {
        match self.album_id.cmp(&other.album_id) {
            Ordering::Equal => {
                if self.track_id.is_none() {
                    Ordering::Less
                } else if other.track_id.is_none() {
                    Ordering::Greater
                } else {
                    match || -> Result<Ordering> {
                        Ok(match column {
                            AudioFileColumn::Title => {
                                match self.disc_number()?.cmp(&other.disc_number()?) {
                                    Ordering::Equal => match self.number()?.cmp(&other.number()?) {
                                        Ordering::Equal => self.title()?.cmp(&other.title()?),
                                        Ordering::Less => Ordering::Less,
                                        Ordering::Greater => Ordering::Greater,
                                    },
                                    Ordering::Less => Ordering::Less,
                                    Ordering::Greater => Ordering::Greater,
                                }
                            }
                            AudioFileColumn::Artist => self.artist()?.cmp(&other.artist()?),
                            AudioFileColumn::Length => self.length()?.cmp(&other.length()?),
                        })
                    }() {
                        Ok(o) => o,
                        Err(e) => {
                            error!("Error comparing album views: {e}!");
                            Ordering::Less
                        }
                    }
                }
            }
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}
