use super::{AudioFile, TagFieldType, Track};
use crate::infrastructure::{musicbrainz::Release, Am};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Album {
    pub artist: String,
    pub artist_id: String,
    pub asin: Option<String>,
    pub barcode: Option<String>,
    pub catalog_number: Option<String>,
    pub date: String,
    pub id: String,
    pub length: Duration,
    pub record_label: Option<String>,
    pub release_country: String,
    pub release_group_id: String,
    pub release_status: String,
    pub release_type: Option<String>,
    pub script: Option<String>,
    pub sort_artist: String,
    pub title: String,
    pub tracks: Vec<Track>,
    pub total_discs: usize,
}

impl Album {
    pub fn match_file(&mut self, audio_file: &Am<AudioFile>) {
        let recording_id = audio_file
            .lock()
            .unwrap()
            .recording_id
            .as_ref()
            .unwrap()
            .clone();
        let track: &mut Track = self
            .tracks
            .iter_mut()
            .find(|t| t.recording_id == recording_id)
            .unwrap();
        track.matched_files.push(audio_file.clone());
    }

    pub fn track(&self, id: &str) -> &Track {
        self.tracks.iter().find(|t| t.id == id).unwrap()
    }

    pub fn update_tag_fields(&self, audio_file: &Am<AudioFile>) {
        let mut audio_file = audio_file.lock().unwrap();
        let acoust_id = audio_file.acoust_id.clone();
        let recording_id = audio_file.recording_id.as_ref().unwrap().clone();
        let track = self
            .tracks
            .iter()
            .find(|t| t.recording_id == *recording_id)
            .unwrap();
        for tag in &mut audio_file.tags {
            tag.set_new_text_value(TagFieldType::Title, track.title.clone());
            tag.set_new_text_value(TagFieldType::Artist, track.artist.clone());
            tag.set_new_text_value(TagFieldType::Album, self.title.clone());
            tag.set_new_text_value(TagFieldType::TrackNumber, track.number.to_string());
            tag.set_new_text_value(TagFieldType::Date, self.date.clone());
            tag.set_new_text_value(TagFieldType::AlbumArtist, self.artist.clone());
            tag.set_new_text_value(TagFieldType::AlbumArtistSortOrder, self.sort_artist.clone());
            tag.set_new_text_value(TagFieldType::ArtistSortOrder, track.sort_artist.clone());
            tag.set_new_text_value(TagFieldType::DiscNumber, track.disc_number.to_string());
            tag.set_new_text_value(TagFieldType::DiscSubtitle, track.disc_subtitle.clone());
            tag.set_new_text_value(TagFieldType::Media, track.media.clone());
            tag.set_new_text_value(TagFieldType::MusicBrainzArtistId, track.artist_id.clone());
            tag.set_new_text_value(TagFieldType::MusicBrainzRecordingId, recording_id.clone());
            tag.set_new_text_value(
                TagFieldType::MusicBrainzReleaseArtistId,
                self.artist_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldType::MusicBrainzReleaseGroupId,
                self.release_group_id.clone(),
            );
            tag.set_new_text_value(TagFieldType::MusicBrainzReleaseId, self.id.clone());
            tag.set_new_text_value(TagFieldType::MusicBrainzTrackId, track.id.clone());
            tag.set_new_text_value(
                TagFieldType::OriginalReleaseDate,
                track.original_release_date.clone(),
            );
            tag.set_new_text_value(TagFieldType::OriginalYear, track.original_year.clone());
            tag.set_new_text_value(TagFieldType::ReleaseCountry, self.release_country.clone());
            tag.set_new_text_value(TagFieldType::ReleaseStatus, self.release_status.clone());
            tag.set_new_text_value(TagFieldType::TotalDiscs, self.total_discs.to_string());
            tag.set_new_text_value(TagFieldType::TotalTracks, track.total_tracks.to_string());

            if let Some(acoust_id) = &acoust_id {
                tag.set_new_text_value(TagFieldType::AcoustId, acoust_id.clone());
            }
            if let Some(asin) = &self.asin {
                tag.set_new_text_value(TagFieldType::Asin, asin.clone());
            }
            if let Some(barcode) = &self.barcode {
                tag.set_new_text_value(TagFieldType::Barcode, barcode.clone());
            }
            if let Some(catalog_number) = &self.catalog_number {
                tag.set_new_text_value(TagFieldType::CatalogNumber, catalog_number.clone());
            }
            if let Some(isrc) = &track.isrc {
                tag.set_new_text_value(TagFieldType::Isrc, isrc.clone());
            }

            if let Some(record_label) = &self.record_label {
                tag.set_new_text_value(TagFieldType::RecordLabel, record_label.clone());
            }
            if let Some(release_type) = &self.release_type {
                tag.set_new_text_value(TagFieldType::ReleaseType, release_type.clone());
            }
            if let Some(script) = &self.script {
                tag.set_new_text_value(TagFieldType::Script, script.clone());
            }
        }
    }
}

impl From<&Release> for Album {
    fn from(release: &Release) -> Self {
        let tracks: Vec<Track> = release
            .media
            .iter()
            .flat_map(|media| media.tracks.iter().map(|track| Track::new(media, track)))
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
            id: release.id.clone(),
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
            tracks,
        }
    }
}
