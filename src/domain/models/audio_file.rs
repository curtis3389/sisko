use super::{Album, Tag, TagFieldId, TagFieldType, Track, TrackId};
use crate::infrastructure::acoustid::Fingerprint;
use crate::infrastructure::file::File;
use crate::infrastructure::{EntityId, Value};
use crate::{domain::events::DomainEvent, infrastructure::Entity};
use anyhow::{anyhow, Result};
use std::cmp::{Eq, PartialEq};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioFileId {
    pub path: PathBuf,
}

impl AudioFileId {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl EntityId for AudioFileId {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<&AudioFileId> for String {
    fn from(id: &AudioFileId) -> Self {
        id.path.to_string_lossy().to_string()
    }
}

impl From<AudioFileId> for String {
    fn from(id: AudioFileId) -> Self {
        String::from(&id)
    }
}

impl Value for AudioFileId {}

/// Represents a file that contains audio data recognized by sisko.
#[derive(Clone, Debug)]
pub struct AudioFile {
    pub acoust_id: Option<String>,

    pub events: Vec<DomainEvent>,

    pub fingerprint: Option<Fingerprint>,

    pub id: AudioFileId,

    pub recording_id: Option<String>,

    pub track_id: Option<TrackId>,
}

impl AudioFile {
    /// Returns a new audio file for the given file and tags.
    ///
    /// # Arguments
    ///
    /// * `file` - The file that contains audio data.
    pub fn new(file: File, fingerprint: Option<Fingerprint>) -> Self {
        let id = AudioFileId::new(file.absolute_path.clone());
        Self {
            acoust_id: None,
            events: vec![],
            fingerprint,
            id,
            recording_id: None,
            track_id: None,
        }
    }

    /*/// Returns the artist of the audio, if any.
    pub fn artist(&self) -> Option<String> {
        // self.tags.iter().filter_map(|t| t.artist()).next()
        todo!()
    }*/

    /// Returns the length of the audio, if any.
    pub fn length(&self) -> Option<String> {
        match &self.fingerprint {
            None => None,
            Some(f) => f.duration.parse::<u64>().ok().map(to_length_string),
        }
    }

    pub fn match_to_album(&mut self, album: &Album, tracks: &[Track]) -> Result<Track> {
        if let Some(recording_id) = &self.recording_id {
            if let Some(track) = tracks.iter().find(|t| t.recording_id == *recording_id) {
                self.track_id = Some(track.id.clone());
                return Ok(track.clone());
            }
        }

        Err(anyhow!(
            "Failed to find a matching track for {} in {}!",
            self.id.to_string(),
            album.id.to_string()
        ))
    }

    pub fn update_tags(&self, album: &Album, track: &Track, mut tags: Vec<Tag>) -> Vec<Tag> {
        let acoust_id = self.acoust_id.clone();
        for tag in &mut tags {
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::Title),
                track.title.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::Artist),
                track.artist.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::Album),
                album.title.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::TrackNumber),
                track.number.to_string(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::Date),
                album.date.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::AlbumArtist),
                album.artist.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::AlbumArtistSortOrder),
                album.sort_artist.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::ArtistSortOrder),
                track.sort_artist.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::DiscNumber),
                track.disc_number.to_string(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::DiscSubtitle),
                track.disc_subtitle.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::Media),
                track.media.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzArtistId),
                track.artist_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzRecordingId),
                track.recording_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzReleaseArtistId),
                album.artist_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzReleaseGroupId),
                album.release_group_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzReleaseId),
                album.id.value.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::MusicBrainzTrackId),
                track.id.track_id.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::OriginalReleaseDate),
                track.original_release_date.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::OriginalYear),
                track.original_year.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::ReleaseCountry),
                album.release_country.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::ReleaseStatus),
                album.release_status.clone(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::TotalDiscs),
                album.total_discs.to_string(),
            );
            tag.set_new_text_value(
                TagFieldId::new(tag.id.clone(), TagFieldType::TotalTracks),
                track.total_tracks.to_string(),
            );

            if let Some(acoust_id) = &acoust_id {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::AcoustId),
                    acoust_id.clone(),
                );
            }
            if let Some(asin) = &album.asin {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::Asin),
                    asin.clone(),
                );
            }
            if let Some(barcode) = &album.barcode {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::Barcode),
                    barcode.clone(),
                );
            }
            if let Some(catalog_number) = &album.catalog_number {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::CatalogNumber),
                    catalog_number.clone(),
                );
            }
            if let Some(isrc) = &track.isrc {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::Isrc),
                    isrc.clone(),
                );
            }

            if let Some(record_label) = &album.record_label {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::RecordLabel),
                    record_label.clone(),
                );
            }
            if let Some(release_type) = &album.release_type {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::ReleaseType),
                    release_type.clone(),
                );
            }
            if let Some(script) = &album.script {
                tag.set_new_text_value(
                    TagFieldId::new(tag.id.clone(), TagFieldType::Script),
                    script.clone(),
                );
            }

            tag.events.push(DomainEvent::TagUpdated(tag.clone()));
        }

        tags
    }

    /*/// Returns the title of the title, if any.
    pub fn title(&self) -> Option<String> {
        // self.tags.iter().filter_map(|t| t.title()).next()
        todo!()
    }

    /// Updates the field of a tag for the given tag type and field data.
    ///
    /// # Arguments
    ///
    /// * `tag_type` - The type of the tag to update the field of.
    /// * `tag_field` - The tag field data to update with.
    pub fn update_tag_field(&mut self, tag_type: &TagType, tag_field: TagField) -> Result<()> {
        /*let tag = self
            .tags
            .iter_mut()
            .find(|t| t.tag_type == *tag_type)
            .ok_or_else(|| {
                anyhow!(
                    "Error trying to update {} tag that's not in the audio file!",
                    tag_type
                )
            })?;
        tag.update_field(tag_field);
        Ok(())*/
        todo!()
    }*/
}

impl Entity for AudioFile {
    type Id = AudioFileId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Eq for AudioFile {}

impl From<&File> for AudioFile {
    fn from(file: &File) -> Self {
        AudioFile::new(file.clone(), None)
    }
}

impl PartialEq for AudioFile {
    fn eq(&self, other: &Self) -> bool {
        // exclude events
        self.acoust_id == other.acoust_id
            && self.fingerprint == other.fingerprint
            && self.id == other.id
            && self.recording_id == other.recording_id
            && self.track_id == other.track_id
    }
}

const SECONDS_IN_HOUR: u64 = 3600;
const SECONDS_IN_MINUTE: u64 = 60;

fn to_length_string(seconds: u64) -> String {
    let hours = seconds / SECONDS_IN_HOUR;
    let hour_part = match hours == 0 {
        true => String::new(),
        false => format!("{}:", hours),
    };
    let seconds = seconds % SECONDS_IN_HOUR;
    let minutes = seconds / SECONDS_IN_MINUTE;
    let seconds = seconds % SECONDS_IN_MINUTE;
    format!("{}{}:{}", hour_part, minutes, seconds)
}
