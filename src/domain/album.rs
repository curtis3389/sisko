use super::Track;
use crate::infrastructure::musicbrainz::Release;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: Duration,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn track(&self, id: &str) -> &Track {
        self.tracks.iter().find(|t| t.id == id).unwrap()
    }
}

impl From<&Release> for Album {
    fn from(release: &Release) -> Self {
        let tracks: Vec<Track> = release
            .media
            .iter()
            .flat_map(|media| media.tracks.iter().map(|track| Track::new(media, track)))
            .collect();
        Self {
            id: release.id.clone(),
            title: release.title.clone(),
            artist: release
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: tracks.iter().map(|t| t.length).sum(),
            tracks,
        }
    }
}
