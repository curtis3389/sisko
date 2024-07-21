use super::Track;
use crate::infrastructure::musicbrainz;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: String,
    pub release_ids: Vec<String>,
    pub tracks: Vec<Arc<Mutex<Track>>>,
}

impl From<&musicbrainz::Recording> for Recording {
    fn from(r: &musicbrainz::Recording) -> Self {
        let length = r.length.map(|l| l / 1000).unwrap_or_default();
        Self {
            id: r.id.clone(),
            title: r.title.clone(),
            artist: r
                .artist_credit
                .get(0)
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: format!("{}:{}", length / 60, length % 60),
            release_ids: r.releases.iter().map(|e| e.id.clone()).collect(),
            tracks: vec![],
        }
    }
}
