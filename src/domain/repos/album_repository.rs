use crate::domain::models::Album;
use crate::domain::services::MediatorService;
use crate::domain::{events::DomainEvent, models::AlbumId};
use crate::infrastructure::database::Database;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use rusqlite::{named_params, Error, Row};
use std::{sync::OnceLock, time::Duration};

pub struct AlbumRepository {}

impl AlbumRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AlbumRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {}
    }

    pub async fn add(&self, album: Album) -> Result<()> {
        let events = album.events.clone();
        Self::insert(album).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Album> {
        Self::select(id).await
    }

    pub async fn get_all(&self) -> Result<Vec<Album>> {
        Self::select_all().await
    }

    pub async fn remove(&self, album: Album) -> Result<()> {
        let events = album.events.clone();
        Self::delete(album).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn save(&self, album: Album) -> Result<()> {
        let events = album.events.clone();
        Self::update(album).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    async fn delete(album: Album) -> Result<()> {
        let id = album.id.value;
        const COMMAND: &str = r#"
            DELETE FROM albums
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| connection.execute(COMMAND, named_params! {":id": id}))
            .await?;
        Ok(())
    }

    async fn insert(album: Album) -> Result<()> {
        const COMMAND: &str = r#"
            INSERT INTO albums (
                id,
                artist,
                artist_id,
                asin,
                barcode,
                catalog_number,
                date,
                length,
                record_label,
                release_country,
                release_group_id,
                release_status,
                release_type,
                script,
                sort_artist,
                title,
                total_discs)
            VALUES (
                :id,
                :artist,
                :artist_id,
                :asin,
                :barcode,
                :catalog_number,
                :date,
                :length,
                :record_label,
                :release_country,
                :release_group_id,
                :release_status,
                :release_type,
                :script,
                :sort_artist,
                :title,
                :total_discs)
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":id": album.id.value,
                        ":artist": album.artist,
                        ":artist_id": album.artist_id,
                        ":asin": album.asin,
                        ":barcode": album.barcode,
                        ":catalog_number": album.catalog_number,
                        ":date": album.date,
                        ":length": album.length.as_secs_f64(),
                        ":record_label": album.record_label,
                        ":release_country": album.release_country,
                        ":release_group_id": album.release_group_id,
                        ":release_status": album.release_status,
                        ":release_type": album.release_type,
                        ":script": album.script,
                        ":sort_artist": album.sort_artist,
                        ":title": album.title,
                        ":total_discs": album.total_discs,
                    },
                )
            })
            .await?;
        Ok(())
    }

    async fn select(id: &str) -> Result<Album> {
        let id = id.to_owned();
        const COMMAND: &str = r#"
            SELECT
                id,
                artist,
                artist_id,
                asin,
                barcode,
                catalog_number,
                date,
                length,
                record_label,
                release_country,
                release_group_id,
                release_status,
                release_type,
                script,
                sort_artist,
                title,
                total_discs
            FROM albums
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Album> {
                let mut statement = connection.prepare(COMMAND)?;
                let album = statement
                    .query_map(named_params! {":id": id}, |row| Album::try_from(row))?
                    .next()
                    .ok_or_else(|| anyhow!("Failed to find album with id {}!", id))??;
                Ok(album)
            })
            .await
    }

    async fn select_all() -> Result<Vec<Album>> {
        const COMMAND: &str = r#"
            SELECT
                id,
                artist,
                artist_id,
                asin,
                barcode,
                catalog_number,
                date,
                length,
                record_label,
                release_country,
                release_group_id,
                release_status,
                release_type,
                script,
                sort_artist,
                title,
                total_discs
            FROM albums
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<Album>> {
                let mut statement = connection.prepare(COMMAND)?;
                let albums: Vec<Album> = statement
                    .query_map((), |row| Album::try_from(row))?
                    .try_collect()?;
                Ok(albums)
            })
            .await
    }

    async fn update(album: Album) -> Result<()> {
        const COMMAND: &str = r#"
            UPDATE albums
            SET artist = :artist,
                artist_id = :artist_id,
                asin = :asin,
                barcode = :barcode,
                catalog_number = :catalog_number,
                date = :date,
                length = :length,
                record_label = :record_label,
                release_country = :release_country,
                release_group_id = :release_group_id,
                release_status = :release_status,
                release_type = :release_type,
                script = :script,
                sort_artist = :sort_artist,
                title = :title,
                total_discs = :total_discs
            WHERE id = :id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":artist": album.artist,
                        ":artist_id": album.artist_id,
                        ":asin": album.asin,
                        ":barcode": album.barcode,
                        ":catalog_number": album.catalog_number,
                        ":date": album.date,
                        ":length": album.length.as_secs_f64(),
                        ":record_label": album.record_label,
                        ":release_country": album.release_country,
                        ":release_group_id": album.release_group_id,
                        ":release_status": album.release_status,
                        ":release_type": album.release_type,
                        ":script": album.script,
                        ":sort_artist": album.sort_artist,
                        ":title": album.title,
                        ":total_discs": album.total_discs,
                        ":id": album.id.value,
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

impl Default for AlbumRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TryFrom<&Row<'a>> for Album {
    type Error = Error;

    fn try_from(row: &Row<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        Ok(Album {
            artist: row.get_unwrap(1),
            artist_id: row.get_unwrap(2),
            asin: row.get_unwrap(3),
            barcode: row.get_unwrap(4),
            catalog_number: row.get_unwrap(5),
            date: row.get_unwrap(6),
            events: vec![],
            id: AlbumId::new(row.get_unwrap(0)),
            length: Duration::from_secs_f64(row.get_unwrap(7)),
            record_label: row.get_unwrap(8),
            release_country: row.get_unwrap(9),
            release_group_id: row.get_unwrap(10),
            release_status: row.get_unwrap(11),
            release_type: row.get_unwrap(12),
            script: row.get_unwrap(13),
            sort_artist: row.get_unwrap(14),
            title: row.get_unwrap(15),
            total_discs: row.get_unwrap(16),
        })
    }
}
