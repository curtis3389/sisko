use crate::domain::{AudioFile, File, TagService};
use anyhow::{anyhow, Result};
use log::error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use super::SiskoService;

/// Represents a service for working with audio files.
/// An audio file is a file that contains audio data that sisko recognizes.
pub struct AudioFileService {
    /// The audio files loaded into the service.
    audio_files: Mutex<HashMap<PathBuf, Arc<Mutex<AudioFile>>>>,
}

impl AudioFileService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AudioFileService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new audio file service.
    pub fn new() -> Self {
        AudioFileService {
            audio_files: Mutex::new(HashMap::new()),
        }
    }

    fn get_clone(&self, file: &File) -> Result<Arc<Mutex<AudioFile>>> {
        Ok(self
            .audio_files
            .lock()
            .map_err(|_| anyhow!("Failed to lock audio files mutex!"))?
            .get(&file.absolute_path)
            .ok_or(anyhow!(
                "File not found in audio files: {}!",
                file.absolute_path.to_string_lossy()
            ))?
            .clone())
    }

    fn is_loaded(&self, file: &File) -> Result<bool> {
        Ok(self
            .audio_files
            .lock()
            .map_err(|_| anyhow!("Error locking audio files mutex!"))?
            .contains_key(&file.absolute_path))
    }

    fn load(&self, file: &File) -> Result<()> {
        let tags = TagService::instance().get_all(file);
        let audio_file = AudioFile::new(file.clone(), None, tags);
        let audio_file = Arc::new(Mutex::new(audio_file));
        let copy = audio_file.clone();
        tokio::spawn(async move {
            if let Err(e) = SiskoService::instance().calculate_fingerprint(&copy) {
                error!("{}", e);
            }
        });
        self.audio_files
            .lock()
            .map_err(|_| anyhow!("Failed to lock audio files mutex!"))?
            .insert(file.absolute_path.clone(), audio_file.clone());
        Ok(())
    }

    pub fn get(&self, file: &File) -> Result<Arc<Mutex<AudioFile>>> {
        if !self.is_loaded(file)? {
            self.load(file)?;
        }
        self.get_clone(file)
    }
}

impl Default for AudioFileService {
    fn default() -> Self {
        Self::new()
    }
}
