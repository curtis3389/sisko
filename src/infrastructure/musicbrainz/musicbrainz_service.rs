use super::{IMusicBrainzService, Recording, Release};
use anyhow::Result;
use reqwest::header::USER_AGENT;
use syrette::injectable;

pub struct MusicBrainzService {}

#[injectable(IMusicBrainzService)]
impl MusicBrainzService {
    fn new() -> Self {
        Self {}
    }
}

impl IMusicBrainzService for MusicBrainzService {
    fn lookup_recording(&self, recording_id: &str) -> Result<Recording> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/recording/{}?inc=releases+artists&fmt=json",
                recording_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()?;
        Ok(response.json()?)
    }

    fn lookup_release(&self, release_id: &str) -> Result<Release> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(format!(
                "https://musicbrainz.org/ws/2/release/{}?inc=artists+recordings&fmt=json",
                release_id
            ))
            .header(USER_AGENT, "sisko/0.1 (curtis.hollibaugh@protonmail.ch)")
            .send()?;
        Ok(response.json()?)
    }
}
