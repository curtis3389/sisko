use crate::domain::AudioFile;
use crate::ui::AudioFileColumn;
use anyhow::{anyhow, Result};
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Represents the UI view of a file that contains audio data..
#[derive(Clone, Debug)]
pub struct AudioFileView {
    /// The title of the audio.
    pub title: String,

    /// The artist of the audio.
    pub artist: String,

    /// The length of the audio.
    pub length: String,

    /// The audio file.
    pub audio_file: Arc<Mutex<AudioFile>>,

    pub path: PathBuf,
}

impl TryFrom<&Arc<Mutex<AudioFile>>> for AudioFileView {
    type Error = anyhow::Error;

    fn try_from(audio_file: &Arc<Mutex<AudioFile>>) -> Result<Self> {
        let mutex = audio_file.clone();
        let audio_file = audio_file
            .lock()
            .map_err(|_| anyhow!("Failed to lock audio file mutex!"))?;
        Ok(Self {
            title: audio_file.title().unwrap_or("<no title>".to_string()),
            artist: audio_file.artist().unwrap_or("<no artist>".to_string()),
            length: audio_file.length().unwrap_or("?:??".to_string()),
            audio_file: mutex,
            path: audio_file.file.absolute_path.clone(),
        })
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
            AudioFileColumn::Title => self.title.to_string(),
            AudioFileColumn::Artist => self.artist.to_string(),
            AudioFileColumn::Length => self.length.to_string(),
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
