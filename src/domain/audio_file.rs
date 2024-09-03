use super::{DomainEvent, File, Tag, TagField, TagService, TagType};
use crate::infrastructure::acoustid::Fingerprint;
use anyhow::{anyhow, Result};

/// Represents a file that contains audio data recognized by sisko.
#[derive(Clone, Debug)]
pub struct AudioFile {
    pub acoust_id: Option<String>,

    pub events: Vec<DomainEvent>,

    /// The file that contains audio data.
    pub file: File,

    pub fingerprint: Option<Fingerprint>,

    pub recording_id: Option<String>,

    /// The metadata tags on the file.
    pub tags: Vec<Tag>,
}

impl AudioFile {
    /// Returns a new audio file for the given file and tags.
    ///
    /// # Arguments
    ///
    /// * `file` - The file that contains audio data.
    /// * `tags` - The metadata tags from the file.
    pub fn new(file: File, fingerprint: Option<Fingerprint>, tags: Vec<Tag>) -> Self {
        Self {
            acoust_id: None,
            events: vec![],
            file,
            fingerprint,
            recording_id: None,
            tags,
        }
    }

    /// Returns the artist of the audio, if any.
    pub fn artist(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.artist()).next()
    }

    /// Returns the length of the audio, if any.
    pub fn length(&self) -> Option<String> {
        match &self.fingerprint {
            None => None,
            Some(f) => f.duration.parse::<u64>().ok().map(to_length_string),
        }
    }

    /// Returns the title of the title, if any.
    pub fn title(&self) -> Option<String> {
        self.tags.iter().filter_map(|t| t.title()).next()
    }

    /// Updates the field of a tag for the given tag type and field data.
    ///
    /// # Arguments
    ///
    /// * `tag_type` - The type of the tag to update the field of.
    /// * `tag_field` - The tag field data to update with.
    pub fn update_tag_field(&mut self, tag_type: &TagType, tag_field: TagField) -> Result<()> {
        let tag = self
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
        Ok(())
    }
}

impl From<&File> for AudioFile {
    fn from(file: &File) -> Self {
        let tags = TagService::instance().get_all(file);
        AudioFile::new(file.clone(), None, tags)
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
