use super::Entity;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MediaTrack {
    pub length: Option<u32>,
    pub id: String,
    pub number: String,
    pub title: String,
    pub recording: Entity,
    pub position: i32,
}
