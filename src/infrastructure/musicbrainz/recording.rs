use super::ArtistCredit;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Recording {
    pub title: String,
    pub id: String,
    pub length: Option<u64>,
    #[serde(rename = "first-release-date")]
    pub first_release_date: String,
    pub video: bool,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    pub disambiguation: String,
    pub isrcs: Vec<String>,
}
