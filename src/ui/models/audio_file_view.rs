use super::AudioFileColumn;
use crate::domain::models::{AudioFile, AudioFileId, Tag, Tags};
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;

/// Represents the UI view of a file that contains audio data..
#[derive(Clone, Debug)]
pub struct AudioFileView {
    pub artist: String,
    pub id: AudioFileId,
    pub length: String,
    pub title: String,
}

impl AudioFileView {
    pub fn new(audio_file: &AudioFile, tags: &Vec<Tag>) -> Self {
        Self {
            artist: tags.artist().unwrap_or("<no artist>".to_string()),
            id: audio_file.id.clone(),
            length: audio_file.length().unwrap_or("?:??".to_string()),
            title: tags.title().unwrap_or("<no title>".to_string()),
        }
    }
}

impl TableViewItem<AudioFileColumn> for AudioFileView {
    /// Returns the value of the given column for this AudioFile.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: AudioFileColumn) -> String {
        match column {
            AudioFileColumn::Title => self.title.clone(),
            AudioFileColumn::Artist => self.artist.clone(),
            AudioFileColumn::Length => self.length.clone(),
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
        match column {
            AudioFileColumn::Title => self.title.cmp(&other.title),
            AudioFileColumn::Artist => self.artist.cmp(&other.artist),
            AudioFileColumn::Length => self.length.cmp(&other.length),
        }
    }
}
