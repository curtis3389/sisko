use super::MediaTrack;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Media {
    #[serde(rename = "format-id")]
    pub format_id: String,
    pub position: i32,
    #[serde(rename = "track-count")]
    pub track_count: i32,
    #[serde(rename = "track-offset")]
    pub track_offset: i32,
    pub tracks: Vec<MediaTrack>,
    pub title: String,
    pub format: String,
}
