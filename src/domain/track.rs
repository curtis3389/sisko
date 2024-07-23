use super::AudioFile;
use crate::infrastructure::{musicbrainz, Am};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: Duration,
    pub number: i32,
    pub disc_number: i32,
    pub matched_files: Vec<Am<AudioFile>>,
    pub recording_id: String,
}

impl Track {
    pub fn new(media: &musicbrainz::Media, track: &musicbrainz::MediaTrack) -> Self {
        let length = track.length.map(Duration::from_millis).unwrap_or_default();
        Self {
            id: track.id.clone(),
            title: track.title.clone(),
            artist: track
                .recording
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length,
            number: track.number.parse::<i32>().unwrap_or_default(),
            disc_number: media.position,
            matched_files: Vec::new(),
            recording_id: track.recording.id.clone(),
        }
    }
}
