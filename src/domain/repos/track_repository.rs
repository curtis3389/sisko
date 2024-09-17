use crate::domain::events::DomainEvent;
use crate::domain::models::{Album, AlbumId, Track, TrackId};
use crate::domain::services::MediatorService;
use crate::infrastructure::database::Database;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use rusqlite::{named_params, Error, Row};
use std::{sync::OnceLock, time::Duration};

pub struct TrackRepository {}

impl TrackRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<TrackRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {}
    }

    pub async fn add(&self, track: Track) -> Result<()> {
        let events = track.events.clone();
        Self::insert(track).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn add_all(&self, tracks: Vec<Track>) -> Result<()> {
        let events = tracks
            .iter()
            .flat_map(|track| track.events.clone())
            .collect();
        Self::insert_all(tracks).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn get(&self, album: &Album, id: &str) -> Result<Track> {
        self.get_by_key(&TrackId::new(album.id.clone(), String::from(id)))
            .await
    }

    pub async fn get_all(&self, album: &Album) -> Result<Vec<Track>> {
        Self::select_all(&album.id.value).await
    }

    pub async fn get_by_key(&self, track_id: &TrackId) -> Result<Track> {
        Self::select(&track_id.album_id.value, &track_id.track_id).await
    }

    pub async fn remove(&self, track: Track) -> Result<()> {
        let events = track.events.clone();
        Self::delete(track).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn save(&self, track: Track) -> Result<()> {
        let events = track.events.clone();
        Self::update(track).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    async fn delete(track: Track) -> Result<()> {
        let album_id = track.id.album_id.value;
        let id = track.id.track_id;
        const COMMAND: &str = r#"
            DELETE FROM tracks
            WHERE
                album_id = :album_id
                AND id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(COMMAND, named_params! {":album_id": album_id, ":id": id})
            })
            .await?;
        Ok(())
    }

    async fn insert(track: Track) -> Result<()> {
        const COMMAND: &str = r#"
            INSERT INTO tracks (
                album_id,
                id,
                artist,
                artist_id,
                disc_number,
                disc_subtitle,
                isrc,
                length,
                media,
                number,
                original_release_date,
                original_year,
                recording_id,
                sort_artist,
                title,
                total_tracks)
            VALUES (
                :album_id,
                :id,
                :artist,
                :artist_id,
                :disc_number,
                :disc_subtitle,
                :isrc,
                :length,
                :media,
                :number,
                :original_release_date,
                :original_year,
                :recording_id,
                :sort_artist,
                :title,
                :total_tracks)
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":album_id": track.id.album_id.value,
                        ":id": track.id.track_id,
                        ":artist": track.artist,
                        ":artist_id": track.artist_id,
                        ":disc_number": track.disc_number,
                        ":disc_subtitle": track.disc_subtitle,
                        ":isrc": track.isrc,
                        ":length": track.length.as_secs_f64(),
                        ":media": track.media,
                        ":number": track.number,
                        ":original_release_date": track.original_release_date,
                        ":original_year": track.original_year,
                        ":recording_id": track.recording_id,
                        ":sort_artist": track.sort_artist,
                        ":title": track.title,
                        ":total_tracks": track.total_tracks,
                    },
                )
            })
            .await?;
        Ok(())
    }

    async fn insert_all(tracks: Vec<Track>) -> Result<()> {
        let values = tracks
            .iter()
            .map(|track| {
                format!("('{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')",
                    track.id.album_id.value,
                    track.id.track_id,
                    track.artist,
                    track.artist_id,
                    track.disc_number,
                    track.disc_subtitle,
                    track.isrc.as_ref().map(|s| format!("'{}'", s)).unwrap_or(String::from("NULL")),
                    track.length.as_secs_f64(),
                    track.media,
                    track.number,
                    track.original_release_date,
                    track.original_year,
                    track.recording_id,
                    track.sort_artist,
                    track.title,
                    track.total_tracks,
                )
            })
            .join(",");
        let command = format!(
            r#"
            INSERT INTO tracks (
                album_id,
                id,
                artist,
                artist_id,
                disc_number,
                disc_subtitle,
                isrc,
                length,
                media,
                number,
                original_release_date,
                original_year,
                recording_id,
                sort_artist,
                title,
                total_tracks)
            VALUES {}
        "#,
            values
        );
        Database::instance()
            .connection
            .call_unwrap(move |connection| connection.execute(&command, ()))
            .await?;
        Ok(())
    }

    async fn select(album_id: &str, track_id: &str) -> Result<Track> {
        let album_id = album_id.to_owned();
        let track_id = track_id.to_owned();
        const COMMAND: &str = r#"
            SELECT
                album_id,
                id,
                artist,
                artist_id,
                disc_number,
                disc_subtitle,
                isrc,
                length,
                media,
                number,
                original_release_date,
                original_year,
                recording_id,
                sort_artist,
                title,
                total_tracks
            FROM tracks
            WHERE
                album_id = :album_id
                AND id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Track> {
                let mut statement = connection.prepare(COMMAND)?;
                let track = statement
                    .query_map(
                        named_params! {":album_id": album_id, ":id": track_id},
                        |row| Track::try_from(row),
                    )?
                    .next()
                    .ok_or_else(|| {
                        anyhow!(
                            "Failed to find track for album {} with id {}!",
                            album_id,
                            track_id
                        )
                    })??;
                Ok(track)
            })
            .await
    }

    async fn select_all(album_id: &str) -> Result<Vec<Track>> {
        let album_id = album_id.to_owned();
        const COMMAND: &str = r#"
            SELECT
                album_id,
                id,
                artist,
                artist_id,
                disc_number,
                disc_subtitle,
                isrc,
                length,
                media,
                number,
                original_release_date,
                original_year,
                recording_id,
                sort_artist,
                title,
                total_tracks
            FROM tracks
            WHERE album_id = :album_id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<Track>> {
                let mut statement = connection.prepare(COMMAND)?;
                let tracks = statement
                    .query_map(named_params! {":album_id": album_id}, |row| {
                        Track::try_from(row)
                    })?
                    .try_collect()?;
                Ok(tracks)
            })
            .await
    }

    async fn update(track: Track) -> Result<()> {
        const COMMAND: &str = r#"
            UPDATE tracks
            SET artist = :artist,
                artist_id = :artist_id,
                disc_number = :disc_number,
                disc_subtitle = :disc_subtitle,
                isrc = :isrc,
                length = :length,
                media = :media,
                number = :number,
                original_release_date = :original_release_date,
                original_year = :original_year,
                recording_id = :recording_id,
                sort_artist = :sort_artist,
                title = :title,
                total_tracks = :total_tracks
            WHERE
                album_id = :album_id
                AND id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":artist": track.artist,
                        ":artist_id": track.artist_id,
                        ":disc_number": track.disc_number,
                        ":disc_subtitle": track.disc_subtitle,
                        ":isrc": track.isrc,
                        ":length": track.length.as_secs_f64(),
                        ":media": track.media,
                        ":number": track.number,
                        ":original_release_date": track.original_release_date,
                        ":original_year": track.original_year,
                        ":recording_id": track.recording_id,
                        ":sort_artist": track.sort_artist,
                        ":title": track.title,
                        ":total_tracks": track.total_tracks,
                        ":album_id": track.id.album_id.value,
                        ":id": track.id.track_id,
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

impl Default for TrackRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TryFrom<&Row<'a>> for Track {
    type Error = Error;

    fn try_from(row: &Row<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        Ok(Track {
            artist: row.get_unwrap(2),
            artist_id: row.get_unwrap(3),
            disc_number: row.get_unwrap(4),
            disc_subtitle: row.get_unwrap(5),
            events: vec![],
            id: TrackId::new(AlbumId::new(row.get_unwrap(0)), row.get_unwrap(1)),
            isrc: row.get_unwrap(6),
            length: Duration::from_secs_f64(row.get_unwrap(7)),
            media: row.get_unwrap(8),
            number: row.get_unwrap(9),
            original_release_date: row.get_unwrap(10),
            original_year: row.get_unwrap(11),
            recording_id: row.get_unwrap(12),
            sort_artist: row.get_unwrap(13),
            title: row.get_unwrap(14),
            total_tracks: row.get_unwrap(15),
        })
    }
}
