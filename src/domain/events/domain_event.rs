use crate::domain::models::{AudioFile, AudioFileId, Metadata};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainEvent {
    AudioFileAdded(AudioFile),
    AudioFileRemoved(AudioFileId),
    AudioFileUpdated(AudioFile),
    TagUpdated(Metadata),
}
