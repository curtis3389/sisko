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
    /// The audio file.
    pub audio_file: Arc<Mutex<AudioFile>>,

    pub path: PathBuf,
}

impl AudioFileView {
    pub fn artist(&self) -> String {
        let audio_file = self.audio_file.lock().unwrap();
        audio_file.artist().unwrap_or("<no artist>".to_string())
    }

    pub fn length(&self) -> String {
        let audio_file = self.audio_file.lock().unwrap();
        audio_file.length().unwrap_or("?:??".to_string())
    }

    pub fn title(&self) -> String {
        let audio_file = self.audio_file.lock().unwrap();
        audio_file.title().unwrap_or("<no title>".to_string())
    }
}

impl TryFrom<&Arc<Mutex<AudioFile>>> for AudioFileView {
    type Error = anyhow::Error;

    fn try_from(audio_file: &Arc<Mutex<AudioFile>>) -> Result<Self> {
        let mutex = audio_file.clone();
        let audio_file = audio_file.lock().map_err(|_| anyhow!(""))?;
        Ok(Self {
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
            AudioFileColumn::Title => self.title(),
            AudioFileColumn::Artist => self.artist(),
            AudioFileColumn::Length => self.length(),
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
            AudioFileColumn::Title => self.title().cmp(&other.title()),
            AudioFileColumn::Artist => self.artist().cmp(&other.artist()),
            AudioFileColumn::Length => self.length().cmp(&other.length()),
        }
    }
}
