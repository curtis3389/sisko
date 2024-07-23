use super::Recording;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MediaTrack {
    pub length: Option<u64>,
    pub id: String,
    pub number: String,
    pub title: String,
    pub recording: Recording,
    pub position: i32,
}
