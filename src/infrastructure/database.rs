use anyhow::Result;
use rusqlite::DatabaseName;
use std::sync::OnceLock;
use tokio_rusqlite::Connection;

const DDL: &str = r#"
    CREATE TABLE albums (
        id TEXT PRIMARY KEY,
        artist TEXT NOT NULL,
        artist_id TEXT NOT NULL,
        asin TEXT,
        barcode TEXT,
        catalog_number TEXT,
        date TEXT NOT NULL,
        length REAL NOT NULL,
        record_label TEXT,
        release_country TEXT NOT NULL,
        release_group_id TEXT NOT NULL,
        release_status TEXT NOT NULL,
        release_type TEXT,
        script TEXT,
        sort_artist TEXT NOT NULL,
        title TEXT NOT NULL,
        total_discs INTEGER NOT NULL
    );

    CREATE INDEX idx_albums_artist_id
    ON albums(artist_id);

    CREATE INDEX idx_albums_release_group_id
    ON albums(release_group_id);

    CREATE TABLE tracks (
        album_id TEXT NOT NULL,
        id TEXT NOT NULL,
        artist TEXT NOT NULL,
        artist_id TEXT NOT NULL,
        disc_number INTEGER NOT NULL,
        disc_subtitle TEXT NOT NULL,
        isrc TEXT,
        length REAL NOT NULL,
        media TEXT NOT NULL,
        number INTEGER NOT NULL,
        original_release_date TEXT NOT NULL,
        original_year TEXT NOT NULL,
        recording_id TEXT NOT NULL,
        sort_artist TEXT NOT NULL,
        title TEXT NOT NULL,
        total_tracks INTEGER NOT NULL,
        PRIMARY KEY (album_id, id),
        FOREIGN KEY (album_id) REFERENCES albums(id)
    );

    CREATE INDEX tracks_artist_id
    ON tracks(artist_id);

    CREATE INDEX tracks_recording_id
    ON tracks(recording_id);

    CREATE TABLE audio_files (
        id TEXT PRIMARY KEY,
        acoust_id TEXT,
        duration TEXT,
        fingerprint TEXT,
        recording_id TEXT,
        album_id TEXT,
        track_id TEXT,
        FOREIGN KEY (album_id, track_id) REFERENCES tracks(album_id, id)
    );

    CREATE INDEX idx_audio_files_acoust_id
    ON audio_files(acoust_id);

    CREATE INDEX idx_audio_files_recording_id
    ON audio_files(recording_id);

    CREATE TABLE metadata_fields (
        audio_file_id TEXT NOT NULL,
        field_type TEXT NOT NULL,
        value_discriminator TEXT,
        value TEXT,
        new_value_discriminator TEXT,
        new_value TEXT,
        PRIMARY KEY (audio_file_id, field_type),
        FOREIGN KEY (audio_file_id) REFERENCES audio_files(id)
    );
"#;

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<Database> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        let connection = futures::executor::block_on(Self::create_connection()).unwrap();
        Self { connection }
    }

    async fn create_connection() -> Result<Connection> {
        let connection = Connection::open_in_memory().await?;
        connection
            .call_unwrap(|connection| connection.execute_batch(DDL))
            .await?;
        connection
            .call_unwrap(|connection| connection.backup(DatabaseName::Main, "db.sqlite", None))
            .await?;
        Ok(connection)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
