use super::{Recording, Release};
use anyhow::Result;
use reqwest::header::USER_AGENT;
use std::sync::OnceLock;

pub struct MusicBrainzService {}

impl MusicBrainzService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<MusicBrainzService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {}
    }

    pub async fn lookup_recording(&self, recording_id: &str) -> Result<Recording> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/recording/{}?inc=releases+artists&fmt=json",
                recording_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()
            .await?;
        Ok(response.json().await?)
    }

    pub async fn lookup_release(&self, release_id: &str) -> Result<Release> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/release/{}?inc=artists+recordings&fmt=json",
                release_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()
            .await?;
        Ok(response.json().await?)
    }
}

impl Default for MusicBrainzService {
    fn default() -> Self {
        Self::new()
    }
}
