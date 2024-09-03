use crate::domain::AudioFile;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use super::MediatorService;

/// Represents a service for working with audio files.
/// An audio file is a file that contains audio data that sisko recognizes.
pub struct AudioFileRepository {
    /// The audio files loaded into the service.
    audio_files: RwLock<HashMap<PathBuf, AudioFile>>,
}

impl AudioFileRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AudioFileRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new audio file service.
    pub fn new() -> Self {
        Self {
            audio_files: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, mut audio_file: AudioFile) -> Result<()> {
        let events = audio_file.events;
        audio_file.events = vec![];
        self.audio_files
            .write()
            .unwrap()
            .insert(audio_file.file.absolute_path.clone(), audio_file);
        let mediator_service = MediatorService::instance();
        for event in events {
            mediator_service.publish(&event)?;
        }
        Ok(())
    }

    pub fn get(&self, path: &PathBuf) -> Option<AudioFile> {
        self.audio_files.read().unwrap().get(path).cloned()
    }

    pub fn remove(&self, audio_file: &AudioFile) {
        self.audio_files
            .write()
            .unwrap()
            .remove(&audio_file.file.absolute_path);
    }

    pub fn save(&self, audio_file: AudioFile) -> Result<()> {
        self.add(audio_file)
    }
}

impl Default for AudioFileRepository {
    fn default() -> Self {
        Self::new()
    }
}
