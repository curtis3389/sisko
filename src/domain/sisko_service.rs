use super::{LogHistory, TrackService};
use crate::domain::{AlbumService, File, FileService, Track};
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
    ///
    /// # Arguments
    ///
    /// * `file_service` - A service for files.
    /// * `track_service` - A service for tracks.
    /// * `ui` - A service for the UI.
    pub fn new() -> Self {
        SiskoService {}
    }

    pub fn add_file(&self, file: Arc<File>) -> Result<()> {
        let track = TrackService::instance().get(&file)?;
        UiWrapper::instance().add_cluster_file(track);
        Ok(())
    }

    pub fn add_folder(&self, file: Arc<File>) -> Result<()> {
        let files = FileService::instance().get_files_in_dir_recursive(&file.absolute_path)?;
        let tracks: Vec<Arc<Mutex<Track>>> = files
            .iter()
            .map(|f| TrackService::instance().get(f))
            .try_collect()?;
        tracks
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

    pub fn select_track(&self, track: &Arc<Mutex<Track>>) {
        UiWrapper::instance().set_metadata_table(track);
    }

    pub async fn scan_track(&self, track: &Arc<Mutex<Track>>) -> Result<()> {
        // get fingerprint for audiofile
        // get recording id for fingerprint
        // load metadata for recording id
        // match audiofile to a release
        // add matched release to album table
        let recording = AlbumService::instance()
            .get_recording_for_track(track)
            .await?;
        let album = AlbumService::instance()
            .get_album_for_recording(&recording)
            .await?;
        UiWrapper::instance().add_album(album);
        Ok(())
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
