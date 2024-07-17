use super::{ArtistCredit, CoverArtArchive, Media, ReleaseEvent, TextRepresentation};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Release {
    pub disambiguation: String,
    pub barcode: String,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    pub id: String,
    pub media: Vec<Media>,
    pub packaging: String,
    pub date: String,
    #[serde(rename = "text-representation")]
    pub text_representation: TextRepresentation,
    #[serde(rename = "status-id")]
    pub status_id: String,
    pub asin: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(rename = "packaging-id")]
    pub packaging_id: String,
    #[serde(rename = "cover-art-archive")]
    pub cover_art_archive: CoverArtArchive,
    pub quality: String,
    #[serde(rename = "release-events")]
    pub release_events: Vec<ReleaseEvent>,
    pub country: String,
}
