use super::{AudioFileService, LogHistory};
use crate::domain::{AlbumService, AudioFile, File, FileService};
use crate::ui::UiWrapper;
use anyhow::Result;
use itertools::Itertools;
use std::sync::{Arc, Mutex, OnceLock};

/// Represents a service for application actions.
pub struct SiskoService {}

impl SiskoService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<SiskoService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new service for application actions.
    pub fn new() -> Self {
        SiskoService {}
    }

    pub fn add_file(&self, file: Arc<File>) -> Result<()> {
        let audio_file = AudioFileService::instance().get(&file)?;
        UiWrapper::instance().add_cluster_file(audio_file);
        Ok(())
    }

    pub fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = FileService::instance().get_files_in_dir_recursive(&file.absolute_path)?;
        let audio_files: Vec<Arc<Mutex<AudioFile>>> = files
            .iter()
            .map(|f| AudioFileService::instance().get(f))
            .try_collect()?;
        audio_files
            .into_iter()
            .for_each(|t| UiWrapper::instance().add_cluster_file(t));
        Ok(())
    }

    pub fn open_logs(&self) {
        let logs = LogHistory::instance()
            .logs()
            .lock()
            .unwrap()
            .join("")
            .lines()
            .rev()
            .join("");
        UiWrapper::instance().open_logs(&logs);
    }

    pub fn select_audio_file(&self, audio_file: &Arc<Mutex<AudioFile>>) {
        UiWrapper::instance().set_metadata_table(audio_file);
    }

    pub async fn scan_audio_file(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let album = AlbumService::instance()
            .get_album_for_file(audio_file)
            .await?;
        UiWrapper::instance().remove_cluster_file(audio_file);
        UiWrapper::instance().add_album(album);
        Ok(())
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
