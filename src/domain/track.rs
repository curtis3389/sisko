use super::AudioFile;
use crate::infrastructure::{musicbrainz, Am};

#[derive(Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: String,
    pub number: i32,
    pub disc_number: i32,
    pub matched_files: Vec<Am<AudioFile>>,
    pub recording_id: String,
}

impl Track {
    pub fn new(media: &musicbrainz::Media, track: &musicbrainz::MediaTrack) -> Self {
        let length = track.length.map(|l| l / 1000).unwrap_or_default();
        Self {
            id: track.id.clone(),
            title: track.title.clone(),
            artist: track
                .recording
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: format!("{}:{}", length / 60, length % 60),
            number: track.number.parse::<i32>().unwrap(),
            disc_number: media.position,
            matched_files: Vec::new(),
            recording_id: track.recording.id.clone(),
        }
    }
}
