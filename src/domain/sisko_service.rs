use super::{LogHistory, TrackService};
use crate::domain::{File, FileService, Track};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::musicbrainz::MusicBrainzService;
use crate::ui::UiWrapper;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::{error, info};
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
        let track_path = {
            let track = track
                .lock()
                .map_err(|_| anyhow!("Error locking track mutex!"))?;
            track.file.absolute_path.clone()
        };
        let fingerprint = AcoustIdService::instance().get_fingerprint(&track_path)?;
        let lookup = AcoustIdService::instance()
            .lookup_fingerprint(&fingerprint)
            .await?;
        let recordingid = lookup[0].recordings[0].id.clone();
        let recording = MusicBrainzService::instance()
            .lookup_recording(&recordingid)
            .await?;
        let release_id = &recording.releases[0].id;
        match MusicBrainzService::instance()
            .lookup_release(release_id)
            .await
        {
            Ok(release) => info!("{} {}", release.title, release.media[0].track_count),
            Err(e) => {
                error!("{}", e);
                let root_cause = e.root_cause();
                error!("root cause: {root_cause}");
            }
        }
        // TODO: pair track to musicbrainz
        // when paired, remove track from cluster table and update album row to paired
        Ok(())
    }
}

impl Default for SiskoService {
    fn default() -> Self {
        Self::new()
    }
}
