use super::{Album, AudioFile};
use crate::infrastructure::acoustid::AcoustIdService;
use crate::infrastructure::musicbrainz::{MusicBrainzService, Release, ReleaseLookup};
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex, OnceLock};

pub struct AlbumService {}

impl AlbumService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AlbumService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_album_for_file(
        &self,
        audio_file: &Arc<Mutex<AudioFile>>,
    ) -> Result<Arc<Mutex<Album>>> {
        // fingerprint
        let file_path = {
            let audio_file = audio_file
                .lock()
                .map_err(|_| anyhow!("Error locking audio file mutex!"))?;
            audio_file.file.absolute_path.clone()
        };
        let fingerprint = AcoustIdService::instance().get_fingerprint(&file_path)?;

        // acoustid
        let lookup = AcoustIdService::instance()
            .lookup_fingerprint(&fingerprint)
            .await?;

        // recording
        let recordingid = lookup[0].recordings[0].id.clone();

        // lookup
        let lookup = MusicBrainzService::instance()
            .lookup_releases_for_recording(&recordingid)
            .await?;

        // get album
        let release_id = self.get_release_id(&lookup)?;
        let release = lookup.releases.iter().find(|r| r.id == release_id).unwrap();
        Ok(Arc::new(Mutex::new(Album::from(release))))
    }

    fn get_release_id(&self, lookup: &ReleaseLookup) -> Result<String> {
        /*let (release, _) = lookup
            .releases
            .iter()
            .map(|r| (r, self.get_priority(&r)))
            .max_by(|(_, p1), (_, p2)| p1.total_cmp(p2))
            .unwrap();
        Ok(release.id.clone())*/
        Ok(lookup.releases.first().unwrap().id.clone())
    }

    fn get_priority(&self, release: &Release) -> f64 {
        0.5
    }
}

impl Default for AlbumService {
    fn default() -> Self {
        Self::new()
    }
}
