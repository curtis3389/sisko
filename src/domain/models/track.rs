use super::AlbumId;
use crate::domain::events::DomainEvent;
use crate::infrastructure::{musicbrainz, Entity, EntityId};
use std::cmp::{Eq, PartialEq};
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackId {
    pub album_id: AlbumId,
    pub track_id: String,
}

impl TrackId {
    pub fn new(album_id: AlbumId, track_id: String) -> Self {
        Self { album_id, track_id }
    }
}

impl EntityId for TrackId {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<&TrackId> for String {
    fn from(id: &TrackId) -> Self {
        format!("{}:{}", id.album_id.to_string(), id.track_id)
    }
}

impl From<TrackId> for String {
    fn from(id: TrackId) -> Self {
        String::from(&id)
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub artist: String,
    pub artist_id: String,
    pub disc_number: i32,
    pub disc_subtitle: String,
    pub events: Vec<DomainEvent>,
    pub id: TrackId,
    pub isrc: Option<String>,
    pub length: Duration,
    pub media: String,
    pub number: i32,
    pub original_release_date: String,
    pub original_year: String,
    pub recording_id: String,
    pub sort_artist: String,
    pub title: String,
    pub total_tracks: i32,
}

impl Track {
    pub fn new(
        album_id: &str,
        media: &musicbrainz::Media,
        track: &musicbrainz::MediaTrack,
    ) -> Self {
        let length = track.length.map(Duration::from_millis).unwrap_or_default();
        let track_artist = track.recording.artist_credit.first();
        Self {
            artist: track_artist
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            artist_id: track_artist
                .map(|a| a.artist.id.clone())
                .unwrap_or_default(),
            disc_number: media.position,
            disc_subtitle: media.title.clone(),
            events: vec![],
            id: TrackId::new(AlbumId::new(album_id.to_owned()), track.id.clone()),
            isrc: track.recording.isrcs.first().cloned(),
            length,
            media: media.format.clone(),
            number: track.number.parse::<i32>().unwrap_or_default(),
            original_release_date: track.recording.first_release_date.clone(),
            original_year: track.recording.first_release_date[0..4].to_string(),
            recording_id: track.recording.id.clone(),
            sort_artist: track_artist
                .map(|a| a.artist.sort_name.clone())
                .unwrap_or_default(),
            title: track.title.clone(),
            total_tracks: media.track_count,
        }
    }

    /* pub async fn has_changes(&self) -> Result<bool> {
        for file in self.matched_files().await? {
            if file.tags().await?.iter().any(|tag| {
                tag.fields.iter().any(|field| match field {
                    super::TagField::Binary(_, old, new) => {
                        new.as_ref().map(|v| !v.eq(old)).unwrap_or(false)
                    }
                    super::TagField::Text(_, old, new) => {
                        new.as_ref().map(|v| !v.eq(old)).unwrap_or(false)
                    }
                    super::TagField::Unknown(_, _) => false,
                })
            }) {
                return Ok(true);
            }
        }
        Ok(false)
    } */
}

impl Entity for Track {
    type Id = TrackId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: EntityId,
    {
        &self.id
    }
}

impl Eq for Track {}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.artist == other.artist
            && self.artist_id == other.artist_id
            && self.disc_number == other.disc_number
            && self.disc_subtitle == other.disc_subtitle
            && self.id == other.id
            && self.isrc == other.isrc
            && self.length == other.length
            && self.media == other.media
            && self.number == other.number
            && self.original_release_date == other.original_release_date
            && self.original_year == other.original_year
            && self.recording_id == other.recording_id
            && self.sort_artist == other.sort_artist
            && self.title == other.title
            && self.total_tracks == other.total_tracks
    }
}
