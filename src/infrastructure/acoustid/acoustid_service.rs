use super::{AcoustIdResult, Fingerprint};
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

pub struct AcoustIdService {}

impl AcoustIdService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AcoustIdService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {}
    }

    pub fn get_fingerprint(&self, path: &Path) -> Result<Fingerprint> {
        let output = Command::new("fpcalc").arg(path).output()?;
        let output = String::from_utf8_lossy(&output.stdout);
        let regex = Regex::new(r"DURATION=(.*)\nFINGERPRINT=(.*)").unwrap();
        let captures: Vec<String> = regex
            .captures(&output)
            .unwrap()
            .iter()
            .skip(1)
            .map(|c| String::from(c.unwrap().as_str()))
            .collect();
        Ok(Fingerprint {
            duration: captures[0].clone(),
            fingerprint: captures[1].clone(),
        })
    }

    pub async fn lookup_fingerprint(
        &self,
        fingerprint: &Fingerprint,
    ) -> Result<Vec<AcoustIdResult>> {
        let mut data: HashMap<&str, String> = HashMap::new();
        data.insert("client", "KS7Sc4UiGc".to_string());
        data.insert("meta", "recordingids".to_string());
        data.insert("duration", fingerprint.duration.clone());
        data.insert("fingerprint", fingerprint.fingerprint.clone());
        let client = reqwest::Client::new();
        let response: LookupResponse = client
            .post("https://api.acoustid.org/v2/lookup")
            .form(&data)
            .send()
            .await?
            .json()
            .await?;
        Ok(response.results)
    }
}

impl Default for AcoustIdService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct LookupResponse {
    status: String,
    results: Vec<AcoustIdResult>,
}
