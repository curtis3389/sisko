use crate::domain::events::DomainEvent;
use crate::domain::models::{AlbumId, AudioFile, AudioFileId, Track, TrackId};
use crate::domain::services::MediatorService;
use crate::infrastructure::{acoustid::Fingerprint, database::Database, EntityId};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use rusqlite::{named_params, Error, Row};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Represents a service for working with audio files.
/// An audio file is a file that contains audio data that sisko recognizes.
pub struct AudioFileRepository {}

impl AudioFileRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AudioFileRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new audio file service.
    pub fn new() -> Self {
        Self {}
    }

    pub async fn add(&self, audio_file: AudioFile) -> Result<()> {
        let events = audio_file.events.clone();
        Self::insert(audio_file).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn get(&self, id: &AudioFileId) -> Result<AudioFile> {
        Self::select(&id.path).await
    }

    pub async fn get_all(&self) -> Result<Vec<AudioFile>> {
        Self::select_all().await
    }

    pub async fn get_matched(&self, track: &Track) -> Result<Vec<AudioFile>> {
        Self::select_matched(track).await
    }

    pub async fn remove(&self, audio_file: AudioFile) -> Result<()> {
        let events = audio_file.events.clone();
        Self::delete(audio_file).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn save(&self, audio_file: AudioFile) -> Result<()> {
        let events = audio_file.events.clone();
        Self::update(audio_file).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    async fn delete(audio_file: AudioFile) -> Result<()> {
        let id = audio_file.id;
        const COMMAND: &str = r#"
            DELETE FROM audio_files
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(COMMAND, named_params! {":id": id.to_string()})
            })
            .await?;
        Ok(())
    }

    async fn insert(audio_file: AudioFile) -> Result<()> {
        const COMMAND: &str = r#"
            INSERT INTO audio_files (id, acoust_id, duration, fingerprint, recording_id, album_id, track_id)
            VALUES (:id, :acoust_id, :duration, :fingerprint, :recording_id, :album_id, :track_id)
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                let (duration, fingerprint) = match audio_file.fingerprint {
                    Some(fingerprint) => {
                        (Some(fingerprint.duration), Some(fingerprint.fingerprint))
                    }
                    None => (None, None),
                };
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":id": audio_file.id.to_string(),
                        ":acoust_id": audio_file.acoust_id,
                        ":duration": duration,
                        ":fingerprint": fingerprint,
                        ":recording_id": audio_file.recording_id,
                        ":album_id": audio_file.track_id.clone().map(|id| id.album_id.value),
                        ":track_id": audio_file.track_id.map(|id| id.track_id),
                    },
                )
            })
            .await?;
        Ok(())
    }

    async fn select(path: &Path) -> Result<AudioFile> {
        let path = path.to_path_buf();
        const COMMAND: &str = r#"
            SELECT id, acoust_id, duration, fingerprint, recording_id, album_id, track_id
            FROM audio_files
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<AudioFile> {
                let mut statement = connection.prepare(COMMAND)?;
                let audio_file = statement
                    .query_map(named_params! {":id": path.to_string_lossy()}, |row| {
                        AudioFile::try_from(row)
                    })?
                    .next()
                    .ok_or_else(|| {
                        anyhow!(
                            "Failed to find audio file with id {}!",
                            path.to_string_lossy()
                        )
                    })??;
                Ok(audio_file)
            })
            .await
    }

    async fn select_all() -> Result<Vec<AudioFile>> {
        const COMMAND: &str = r#"
            SELECT
                id,
                acoust_id,
                duration,
                fingerprint,
                recording_id,
                album_id,
                track_id
            FROM audio_files
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<AudioFile>> {
                let mut statement = connection.prepare(COMMAND)?;
                let audio_files: Vec<AudioFile> = statement
                    .query_map((), |row| AudioFile::try_from(row))?
                    .try_collect()?;
                Ok(audio_files)
            })
            .await
    }

    async fn select_matched(track: &Track) -> Result<Vec<AudioFile>> {
        let album_id = track.id.album_id.value.clone();
        let track_id = track.id.track_id.clone();
        const COMMAND: &str = r#"
            SELECT
                id,
                acoust_id,
                duration,
                fingerprint,
                recording_id,
                album_id,
                track_id
            FROM audio_files
            WHERE
                album_id = :album_id
                AND track_id = :track_id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<AudioFile>> {
                let mut statement = connection.prepare(COMMAND)?;
                let audio_files: Vec<AudioFile> = statement
                    .query_map(
                        named_params! {":album_id": album_id, ":track_id": track_id},
                        |row| AudioFile::try_from(row),
                    )?
                    .try_collect()?;
                Ok(audio_files)
            })
            .await
    }

    async fn update(audio_file: AudioFile) -> Result<()> {
        const COMMAND: &str = r#"
            UPDATE audio_files
            SET acoust_id = :acoust_id,
                duration = :duration,
                fingerprint = :fingerprint,
                recording_id = :recording_id,
                album_id = :album_id,
                track_id = :track_id
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                let (duration, fingerprint) = match audio_file.fingerprint {
                    Some(fingerprint) => {
                        (Some(fingerprint.duration), Some(fingerprint.fingerprint))
                    }
                    None => (None, None),
                };
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":acoust_id": audio_file.acoust_id,
                        ":duration": duration,
                        ":fingerprint": fingerprint,
                        ":recording_id": audio_file.recording_id,
                        ":album_id": audio_file.track_id.clone().map(|id| id.album_id.value),
                        ":track_id": audio_file.track_id.map(|id| id.track_id),
                        ":id": audio_file.id.to_string(),
                    },
                )
            })
            .await?;
        Ok(())
    }

    fn publish_events(events: Vec<DomainEvent>) -> Result<()> {
        let mediator_service = MediatorService::instance();
        for event in events {
            mediator_service.publish(&event)?;
        }
        Ok(())
    }
}

impl Default for AudioFileRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TryFrom<&Row<'a>> for AudioFile {
    type Error = Error;

    fn try_from(row: &Row<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        let id: String = row.get_unwrap(0);
        let id = AudioFileId::new(PathBuf::from(id));
        let duration: Option<String> = row.get_unwrap(2);
        let fingerprint: Option<String> = row.get_unwrap(3);
        let fingerprint = match (duration, fingerprint) {
            (Some(duration), Some(fingerprint)) => Some(Fingerprint {
                duration,
                fingerprint,
            }),
            _ => None,
        };
        let album_id: Option<String> = row.get_unwrap(5);
        let track_id: Option<String> = row.get_unwrap(6);
        let track_id = match (album_id, track_id) {
            (Some(album_id), Some(track_id)) => {
                Some(TrackId::new(AlbumId::new(album_id), track_id))
            }
            _ => None,
        };
        Ok(AudioFile {
            acoust_id: row.get_unwrap(1),
            events: vec![],
            fingerprint,
            id,
            recording_id: row.get_unwrap(4),
            track_id,
        })
    }
}
