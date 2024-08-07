use super::{
    ArtistCredit, CoverArtArchive, LabelInfo, Media, ReleaseEvent, ReleaseGroup, TextRepresentation,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Release {
    pub disambiguation: String,
    pub barcode: Option<String>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    pub id: String,
    pub media: Vec<Media>,
    pub packaging: Option<String>,
    pub date: String,
    #[serde(rename = "text-representation")]
    pub text_representation: TextRepresentation,
    #[serde(rename = "status-id")]
    pub status_id: String,
    pub asin: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(rename = "packaging-id")]
    pub packaging_id: Option<String>,
    #[serde(rename = "cover-art-archive")]
    pub cover_art_archive: CoverArtArchive,
    pub quality: String,
    #[serde(rename = "release-events")]
    pub release_events: Vec<ReleaseEvent>,
    pub country: String,
    #[serde(rename = "release-group")]
    pub release_group: ReleaseGroup,
    #[serde(rename = "label-info")]
    pub label_info: Vec<LabelInfo>,
}
