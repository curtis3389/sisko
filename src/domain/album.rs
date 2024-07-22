use super::Track;
use crate::infrastructure::musicbrainz::Release;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: String,
    pub tracks: Vec<Arc<Mutex<Track>>>,
}

impl From<&Release> for Album {
    fn from(release: &Release) -> Self {
        let tracks: Vec<Arc<Mutex<Track>>> = release
            .media
            .iter()
            .flat_map(|media| {
                media
                    .tracks
                    .iter()
                    .map(|track| Arc::new(Mutex::new(Track::from(track))))
            })
            .collect();
        Self {
            id: release.id.clone(),
            title: release.title.clone(),
            artist: release
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: String::from("?:??"),
            tracks,
        }
    }
}
