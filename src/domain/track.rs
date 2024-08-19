use super::AudioFile;
use crate::infrastructure::{musicbrainz, Am};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Track {
    pub artist: String,
    pub artist_id: String,
    pub disc_number: i32,
    pub disc_subtitle: String,
    pub id: String,
    pub isrc: Option<String>,
    pub length: Duration,
    pub matched_files: Vec<Am<AudioFile>>,
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
    pub fn new(media: &musicbrainz::Media, track: &musicbrainz::MediaTrack) -> Self {
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
            id: track.id.clone(),
            isrc: track.recording.isrcs.first().cloned(),
            length,
            matched_files: Vec::new(),
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

    pub fn has_changes(&self) -> bool {
        self.matched_files.iter().any(|file| {
            let file = file.lock().unwrap();
            file.tags.iter().any(|tag| {
                tag.fields.iter().any(|field| match field {
                    super::TagField::Binary(_, old, new) => {
                        new.as_ref().map(|v| !v.eq(old)).unwrap_or(false)
                    }
                    super::TagField::Text(_, old, new) => {
                        new.as_ref().map(|v| !v.eq(old)).unwrap_or(false)
                    }
                    super::TagField::Unknown(_, _) => false,
                })
            })
        })
    }

    pub fn has_match(&self) -> bool {
        !self.matched_files.is_empty()
    }
}
