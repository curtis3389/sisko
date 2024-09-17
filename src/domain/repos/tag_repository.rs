use crate::domain::events::DomainEvent;
use crate::domain::models::{
    AudioFile, AudioFileId, Tag, TagField, TagFieldId, TagFieldType, TagId, TagType,
};
use crate::domain::services::MediatorService;
use crate::infrastructure::database::Database;
use anyhow::Result;
use itertools::Itertools;
use rusqlite::{named_params, Error, Row};
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

    pub async fn add(&self, tag: Tag) -> Result<()> {
        let events = tag.events.clone();
        Self::insert(tag).await?;
        Database::instance()
            .connection
            .call_unwrap(|connection| {
                connection.backup(rusqlite::DatabaseName::Main, "db.sqlite", None)
            })
            .await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn get(&self, audio_file: &AudioFile, tag_type: &TagType) -> Result<Tag> {
        Self::select(&audio_file.id.path, tag_type).await
    }

    pub async fn get_all(&self, audio_file: &AudioFile) -> Result<Vec<Tag>> {
        let mut tags = vec![];
        for tag_type in [
            TagType::FLAC,
            TagType::ID3v1,
            TagType::ID3v2,
            TagType::Vorbis,
        ] {
            if let Ok(tag) = self.get(audio_file, &tag_type).await {
                tags.push(tag);
            }
        }
        Ok(tags)
    }

    pub async fn remove(&self, tag: Tag) -> Result<()> {
        let events = tag.events.clone();
        Self::delete(tag).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    pub async fn save(&self, tag: Tag) -> Result<()> {
        let events = tag.events.clone();
        Self::update(tag).await?;
        Self::publish_events(events)?;
        Ok(())
    }

    async fn delete(tag: Tag) -> Result<()> {
        let audio_file_id = tag.id.audio_file_id.path.clone();
        let tag_type = tag.id.tag_type;
        const COMMAND: &str = r#"
            DELETE FROM tag_fields
            WHERE
                audio_file_id = :audio_file_id
                AND tag_type = :tag_type
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| connection.execute(COMMAND, named_params! {":audio_file_id": audio_file_id.to_string_lossy(), ":tag_type": tag_type.as_str()}))
            .await?;
        Ok(())
    }

    async fn insert(tag: Tag) -> Result<()> {
        for tag_field in tag.fields {
            Self::insert_field(&tag.id.audio_file_id.path, &tag.id.tag_type, tag_field).await?;
        }
        Ok(())
    }

    async fn insert_field(
        audio_file_id: &Path,
        tag_type: &TagType,
        tag_field: TagField,
    ) -> Result<()> {
        let audio_file_id = audio_file_id.to_path_buf();
        let tag_type = *tag_type;
        const COMMAND: &str = r#"
            INSERT INTO tag_fields (
                audio_file_id,
                tag_type,
                tag_field_type,
                discriminator,
                value,
                new_value)
            VALUES (
                :audio_file_id,
                :tag_type,
                :tag_field_type,
                :discriminator,
                :value,
                :new_value)
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| match tag_field {
                TagField::Binary(id, value, new_value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Binary",
                        ":value": value,
                        ":new_value": new_value,
                    },
                ),
                TagField::Text(id, value, new_value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Text",
                        ":value": value,
                        ":new_value": new_value,
                    },
                ),
                TagField::Unknown(id, value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Unknown",
                        ":value": value,
                        ":new_value": None::<String>,
                    },
                ),
            })
            .await?;
        Ok(())
    }

    async fn select(audio_file_id: &Path, tag_type: &TagType) -> Result<Tag> {
        let id = audio_file_id.to_path_buf();
        let ttype = *tag_type;
        const COMMAND: &str = r#"
            SELECT
                audio_file_id,
                tag_type,
                tag_field_type,
                discriminator,
                value,
                new_value
            FROM tag_fields
            WHERE
                audio_file_id = :audio_file_id
                AND tag_type = :tag_type
        "#;
        let fields = Database::instance()
            .connection
            .call_unwrap(move |connection| -> Result<Vec<TagField>> {
                let mut statement = connection.prepare(COMMAND)?;
                let fields: Vec<TagField> = statement
                    .query_map(named_params! {":audio_file_id": id.to_string_lossy(), ":tag_type": ttype.as_str()}, |row| TagField::try_from(row))?.try_collect()?;
                Ok(fields)
            })
            .await?;
        Ok(Tag::new(
            TagId::new(AudioFileId::new(audio_file_id.to_path_buf()), *tag_type),
            fields,
        ))
    }

    async fn update(tag: Tag) -> Result<()> {
        for tag_field in tag.fields {
            Self::upsert_field(&tag.id.audio_file_id.path, &tag.id.tag_type, tag_field).await?;
        }
        Ok(())
    }

    async fn upsert_field(
        audio_file_id: &Path,
        tag_type: &TagType,
        tag_field: TagField,
    ) -> Result<()> {
        let audio_file_id = audio_file_id.to_path_buf();
        let tag_type = *tag_type;
        const COMMAND: &str = r#"
            INSERT INTO tag_fields (
                audio_file_id,
                tag_type,
                tag_field_type,
                discriminator,
                value,
                new_value)
            VALUES (
                :audio_file_id,
                :tag_type,
                :tag_field_type,
                :discriminator,
                :value,
                :new_value)
            ON CONFLICT(audio_file_id, tag_type, tag_field_type)
            DO UPDATE SET
                discriminator = :discriminator,
                value = :value,
                new_value = :new_value
        "#;
        Database::instance()
            .connection
            .call_unwrap(move |connection| match tag_field {
                TagField::Binary(id, value, new_value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Binary",
                        ":value": value,
                        ":new_value": new_value,
                    },
                ),
                TagField::Text(id, value, new_value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Text",
                        ":value": value,
                        ":new_value": new_value,
                    },
                ),
                TagField::Unknown(id, value) => connection.execute(
                    COMMAND,
                    named_params! {
                        ":audio_file_id": audio_file_id.to_string_lossy(),
                        ":tag_type": tag_type.as_str(),
                        ":tag_field_type": id.tag_field_type.display_name(),
                        ":discriminator": "Unknown",
                        ":value": value,
                        ":new_value": None::<String>,
                    },
                ),
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

impl<'a> TryFrom<&Row<'a>> for TagField {
    type Error = Error;

    fn try_from(row: &Row<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        let tag_field_type: String = row.get_unwrap(2);
        let tag_field_type = TagFieldType::from(tag_field_type);
        let discriminator: String = row.get_unwrap(3);
        let tag_type: String = row.get_unwrap(1);
        let tag_type = TagType::from(tag_type.as_str());
        let audio_file_id: String = row.get_unwrap(0);
        let tag_id = TagId::new(AudioFileId::new(PathBuf::from(audio_file_id)), tag_type);
        let id = TagFieldId::new(tag_id, tag_field_type);
        Ok(match discriminator.as_str() {
            "Binary" => TagField::Binary(id, row.get_unwrap(4), row.get_unwrap(5)),
            "Text" => TagField::Text(id, row.get_unwrap(4), row.get_unwrap(5)),
            "Unknown" => TagField::Unknown(id, row.get_unwrap(4)),
            _ => panic!("Unknown tag field discriminator: {}!", discriminator),
        })
    }
}
