use crate::domain::events::DomainEvent;
use crate::domain::models::{
    AudioFile, AudioFileId, FieldValue, Metadata, MetadataField, TagFieldType,
};
use crate::domain::services::MediatorService;
use crate::infrastructure::database::Database;
use anyhow::Result;
use itertools::Itertools;
use rusqlite::{named_params, Row};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Represents a service for working with audio file tags.
pub struct TagRepository {}

impl TagRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<TagRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new tag service.
    pub fn new() -> Self {
        Self {}
    }

    pub async fn add(&self, metadata: Metadata) -> Result<()> {
        let events = metadata.events.clone();
        Self::insert(metadata).await?;
        // TODO: move this to a menu action
        /*Database::instance()
        .connection
        .call_unwrap(|connection| {
            connection.backup(rusqlite::DatabaseName::Main, "db.sqlite", None)
        })
        .await?;*/
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn get(&self, audio_file: &AudioFile) -> Result<Metadata> {
        Self::select(&audio_file.id.path).await
    }

    pub async fn remove(&self, metadata: Metadata) -> Result<()> {
        let events = metadata.events.clone();
        Self::delete(metadata).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn save(&self, metadata: Metadata) -> Result<()> {
        let events = metadata.events.clone();
        Self::update(metadata).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    async fn delete(metadata: Metadata) -> Result<()> {
        let audio_file_id = metadata.audio_file_id.path.clone();
        const COMMAND: &str = r#"
            DELETE FROM metadat_fields
            WHERE audio_file_id = :audio_file_id
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {":audio_file_id": audio_file_id.to_string_lossy()},
                )
            })
            .await?;
        Ok(())
    }

    async fn insert(metadata: Metadata) -> Result<()> {
        let fields = metadata
            .iter()
            .map(|field| {
                (
                    metadata.audio_file_id.path.to_path_buf(),
                    field.field_type.clone(),
                    field.old_value.clone(),
                    field.new_value.clone(),
                )
            })
            .collect_vec();
        for (audio_file_id, field_type, old_value, new_value) in fields {
            Self::insert_field(audio_file_id, field_type, old_value, new_value).await?;
        }
        Ok(())
    }

    async fn insert_field(
        audio_file_id: PathBuf,
        field_type: TagFieldType,
        old_value: Option<FieldValue>,
        new_value: Option<FieldValue>,
    ) -> Result<()> {
        const COMMAND: &str = r#"
            INSERT INTO metadata_fields (
                audio_file_id,
                field_type,
                value_discriminator,
                value,
                new_value_discriminator
                new_value)
            VALUES (
                :audio_file_id,
                :field_type,
                :value_discriminator,
                :value,
                :new_value_discriminator,
                :new_value)
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":field_type": field_type.display_name(),
                        ":value_discriminator": old_value.as_ref().map(|v| v.discriminator()),
                        ":value": old_value,
                        ":new_value_discriminator": new_value.as_ref().map(|v| v.discriminator()),
                        ":new_value": new_value,
                    },
                )
            })
            .await?;
        Ok(())
    }

    async fn select(audio_file_id: &Path) -> Result<Metadata> {
        let id = audio_file_id.to_path_buf();
        const COMMAND: &str = r#"
            SELECT
                audio_file_id,
                field_type,
                value_discriminator,
                value,
                new_value_discriminator,
                new_value
            FROM metadata_fields
            WHERE audio_file_id = :audio_file_id
        "#;
        let fields = Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<MetadataField>> {
                let mut statement = connection.prepare(COMMAND)?;
                let fields: Vec<MetadataField> = statement
                    .query_map(
                        named_params! {":audio_file_id": id.to_string_lossy()},
                        |row| Ok(to_field(row)),
                    )?
                    .try_collect()?;
                Ok(fields)
            })
            .await?;
        Ok(Metadata::from_fields(
            AudioFileId::new(audio_file_id.to_path_buf()),
            fields,
        ))
    }

    async fn update(metadata: Metadata) -> Result<()> {
        let fields = metadata
            .iter()
            .map(|field| {
                (
                    metadata.audio_file_id.path.to_path_buf(),
                    field.field_type.clone(),
                    field.old_value.clone(),
                    field.new_value.clone(),
                )
            })
            .collect_vec();
        for (audio_file_id, field_type, old_value, new_value) in fields {
            Self::upsert_field(audio_file_id, field_type, old_value, new_value).await?;
        }
        Ok(())
    }

    async fn upsert_field(
        audio_file_id: PathBuf,
        field_type: TagFieldType,
        old_value: Option<FieldValue>,
        new_value: Option<FieldValue>,
    ) -> Result<()> {
        const COMMAND: &str = r#"
            INSERT INTO metadata_fields (
                audio_file_id,
                field_type,
                value_discriminator,
                value,
                new_value_discriminator,
                new_value)
            VALUES (
                :audio_file_id,
                :field_type,
                :value_discriminator,
                :value,
                :new_value_discriminator,
                :new_value)
            ON CONFLICT(audio_file_id, field_type)
            DO UPDATE SET
                value_discriminator = :value_discriminator,
                value = :value,
                new_value_discriminator = :new_value_discriminator,
                new_value = :new_value
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| {
                connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":field_type": field_type.display_name(),
                        ":value_discriminator": old_value.as_ref().map(|v| v.discriminator()),
                        ":value": old_value,
                        ":new_value_discriminator": new_value.as_ref().map(|v| v.discriminator()),
                        ":new_value": new_value,
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

impl Default for TagRepository {
    fn default() -> Self {
        Self::new()
    }
}

fn to_field(row: &Row) -> MetadataField {
    let field_type: String = row.get_unwrap(1);
    let field_type = TagFieldType::from(field_type);
    let value_discriminator: Option<String> = row.get_unwrap(2);
    let old_value = to_value(row, 3, value_discriminator);
    let new_value_discriminator: Option<String> = row.get_unwrap(4);
    let new_value = to_value(row, 5, new_value_discriminator);
    MetadataField {
        field_type,
        new_value,
        old_value,
    }
}

fn to_value(row: &Row, index: usize, discriminator: Option<String>) -> Option<FieldValue> {
    match discriminator {
        Some(discriminator) => match discriminator.as_str() {
            "Binary" => Some(FieldValue::Binary(row.get_unwrap(index))),
            "Text" => Some(FieldValue::Text(row.get_unwrap(index))),
            "Unknown" => Some(FieldValue::Unknown),
            _ => panic!("Unknown discriminator: {}!", discriminator),
        },
        None => None,
    }
}
