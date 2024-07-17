use super::{ArtistCredit, Entity};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Recording {
    pub title: String,
    pub id: String,
    pub length: u64,
    #[serde(rename = "first-release-date")]
    pub first_release_date: String,
    pub video: bool,
    pub releases: Vec<Entity>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    pub disambiguation: String,
}
