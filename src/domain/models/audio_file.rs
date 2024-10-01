use super::{Album, Track, TrackId};
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
