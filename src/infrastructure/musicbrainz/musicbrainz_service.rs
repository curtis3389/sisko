use super::{Recording, Release};
use anyhow::Result;
use log::info;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::sync::OnceLock;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{sleep, Duration, Instant};

static ONE_SEC: Duration = Duration::from_secs(1);

pub struct MusicBrainzService {
    last: AsyncMutex<Instant>,
}

impl MusicBrainzService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<MusicBrainzService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            last: AsyncMutex::new(Instant::now()),
        }
    }

    pub async fn lookup_recording(&self, recording_id: &str) -> Result<Recording> {
        let mut last = self.last.lock().await;
        let elapsed = Instant::now() - last.clone();
        if elapsed < ONE_SEC {
            sleep(ONE_SEC - elapsed).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/recording/{}?inc=releases+artists&fmt=json",
                recording_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()
            .await?;
        let text = response.text().await?;
        info!("{}", text);

        *last = Instant::now();

        Ok(serde_json::from_str(&text)?)
    }

    pub async fn lookup_release(&self, release_id: &str) -> Result<Release> {
        let mut last = self.last.lock().await;
        let elapsed = Instant::now() - last.clone();
        if elapsed < ONE_SEC {
            sleep(ONE_SEC - elapsed).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/release/{}?inc=artists+recordings&fmt=json",
                release_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()
            .await?;

        *last = Instant::now();

        Ok(response.json().await?)
    }

    pub async fn lookup_releases_for_recording(
        &self,
        recording_id: &String,
    ) -> Result<ReleaseLookup> {
        let mut last = self.last.lock().await;
        let elapsed = Instant::now() - last.clone();
        if elapsed < ONE_SEC {
            sleep(ONE_SEC - elapsed).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/release?recording={}&fmt=json&inc=recordings+release-groups+media+isrcs+artist-credits",
                recording_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()
            .await?;

        *last = Instant::now();

        Ok(response.json().await?)
    }
}

impl Default for MusicBrainzService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub struct ReleaseLookup {
    #[serde(rename = "release-count")]
    release_count: i32,
    #[serde(rename = "release-offset")]
    release_offset: i32,
    releases: Vec<Release>,
}
