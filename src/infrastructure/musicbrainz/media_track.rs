use super::Entity;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MediaTrack {
    length: u32,
    id: String,
    number: String,
    title: String,
    recording: Entity,
    position: i32,
}
