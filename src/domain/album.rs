use super::Recording;
use crate::infrastructure::musicbrainz::Release;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: String,
    pub recordings: Vec<Arc<Mutex<Recording>>>,
}

impl Album {
    pub fn new(release: Release, recordings: Vec<Arc<Mutex<Recording>>>) -> Self {
        Self {
            id: release.id.clone(),
            title: release.title.clone(),
            artist: release
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: String::from("?:??"),
            recordings,
        }
    }
}
