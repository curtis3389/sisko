use super::{AudioFileService, LogHistory};
use crate::domain::{AlbumService, AudioFile, File, FileService};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::ui::UiWrapper;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File as FsFile;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
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
        UiWrapper::instance().add_cluster_file(audio_file)?;
        Ok(())
    }

    pub fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = FileService::instance().get_files_in_dir_recursive(&file.absolute_path)?;
        let audio_files: Vec<Arc<Mutex<AudioFile>>> = files
            .iter()
            .map(|f| AudioFileService::instance().get(f))
            .try_collect()?;
        for t in audio_files {
            UiWrapper::instance().add_cluster_file(t)?;
        }
        Ok(())
    }

    pub fn calculate_fingerprint(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let path = {
            let audio_file = audio_file.lock().map_err(|_| anyhow!(""))?;
            audio_file.file.absolute_path.clone()
        };
        let fingerprint = AcoustIdService::instance().get_fingerprint(&path).ok();
        let mut audio_file = audio_file.lock().map_err(|_| anyhow!(""))?;
        audio_file.fingerprint = fingerprint.or(audio_file.fingerprint.clone());
        Ok(())
    }

    pub fn open_logs(&self) -> Result<()> {
        let logs = LogHistory::instance()
            .logs()
            .lock()
            .map_err(|_| anyhow!("Error unlocking the logs mutex!"))?
            .join("")
            .lines()
            .rev()
            .join("");
        UiWrapper::instance().open_logs(&logs)?;
        Ok(())
    }

    pub fn save_audio_file(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let audio_file = audio_file.lock().map_err(|_| anyhow!(""))?;
        let audio_bytes = Self::get_audio_bytes(&audio_file.file.absolute_path)?;
        let filename = &audio_file.file.name;
        let tag = audio_file.tags.first().ok_or_else(|| anyhow!(""))?;
        let tag = ID3v2Tag::from(tag);
        let mut bytes = tag.to_bytes();
        bytes.extend(audio_bytes);
        let mut file = FsFile::create(filename)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    fn get_audio_bytes(path: &PathBuf) -> Result<Vec<u8>> {
        let offset = match ID3v2Tag::read_from_path(path).ok() {
            Some(tag) => tag.total_size(),
            None => 0,
        };

        let mut file = FsFile::open(path)?;
        file.seek(SeekFrom::Start(offset.into()))?;
        let mut file_content: Vec<u8> = vec![];
        file.read_to_end(&mut file_content)?;
        Ok(file_content)
    }

    pub fn select_audio_file(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        UiWrapper::instance().set_metadata_table(audio_file)
    }

    pub async fn scan_audio_file(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let album = AlbumService::instance()
            .get_album_for_file(audio_file)
            .await?;
        UiWrapper::instance().remove_cluster_file(audio_file)?;
        UiWrapper::instance().add_album(album)?;
        Ok(())
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
