use super::ArtistCredit;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReleaseGroup {
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    pub disambiguation: String,
    #[serde(rename = "first-release-date")]
    pub first_release_date: String,
    pub id: String,
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
    #[serde(rename = "primary-type-id")]
    pub primary_type_id: Option<String>,
    #[serde(rename = "secondary-types")]
    pub secondary_types: Vec<String>,
    #[serde(rename = "secondary-type-ids")]
    pub secondary_type_ids: Vec<String>,
    pub title: String,
}
