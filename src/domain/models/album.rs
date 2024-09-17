use super::Track;
use crate::domain::events::DomainEvent;
use crate::infrastructure::musicbrainz::Release;
use crate::infrastructure::{Entity, EntityId};
use std::cmp::{Eq, PartialEq};
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumId {
    pub value: String,
}

impl AlbumId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl EntityId for AlbumId {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<&AlbumId> for String {
    fn from(id: &AlbumId) -> Self {
        id.value.clone()
    }
}

impl From<AlbumId> for String {
    fn from(id: AlbumId) -> Self {
        String::from(&id)
    }
}

#[derive(Clone, Debug)]
pub struct Album {
    pub artist: String,
    pub artist_id: String,
    pub asin: Option<String>,
    pub barcode: Option<String>,
    pub catalog_number: Option<String>,
    pub date: String,
    pub events: Vec<DomainEvent>,
    pub id: AlbumId,
    pub length: Duration,
    pub record_label: Option<String>,
    pub release_country: String,
    pub release_group_id: String,
    pub release_status: String,
    pub release_type: Option<String>,
    pub script: Option<String>,
    pub sort_artist: String,
    pub title: String,
    pub total_discs: usize,
}

impl From<&Release> for Album {
    fn from(release: &Release) -> Self {
        let tracks: Vec<Track> = release
            .media
            .iter()
            .flat_map(|media| {
                media
                    .tracks
                    .iter()
                    .map(|track| Track::new(&release.id.clone(), media, track))
            })
            .collect();
        let album_artist = release.artist_credit.first();
        Self {
            artist: album_artist
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            artist_id: album_artist
                .map(|a| a.artist.id.clone())
                .unwrap_or_default(),
            asin: release.asin.clone(),
            barcode: release.barcode.clone(),
            catalog_number: release
                .label_info
                .first()
                .and_then(|i| i.catalog_number.clone()),
            date: release.date.clone(),
            events: vec![],
            id: AlbumId::new(release.id.clone()),
            length: tracks.iter().map(|t| t.length).sum(),
            record_label: release
                .label_info
                .first()
                .and_then(|i| i.label.as_ref().map(|l| l.name.clone())),
            release_country: release.country.clone(),
            release_group_id: release.release_group.id.clone(),
            release_status: release.status.clone(),
            release_type: release.release_group.primary_type.clone(),
            script: release.text_representation.script.clone(),
            sort_artist: album_artist
                .map(|a| a.artist.sort_name.clone())
                .unwrap_or_default(),
            title: release.title.clone(),
            total_discs: release.media.len(),
        }
    }
}

impl Entity for Album {
    type Id = AlbumId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: EntityId,
    {
        &self.id
    }
}

impl Eq for Album {}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.artist == other.artist
            && self.artist_id == other.artist_id
            && self.asin == other.asin
            && self.barcode == other.barcode
            && self.catalog_number == other.catalog_number
            && self.date == other.date
            && self.id == other.id
            && self.length == other.length
            && self.record_label == other.record_label
            && self.release_country == other.release_country
            && self.release_group_id == other.release_group_id
            && self.release_status == other.release_status
            && self.release_type == other.release_type
            && self.script == other.script
            && self.sort_artist == other.sort_artist
            && self.title == other.title
            && self.total_discs == other.total_discs
    }
}
