use crate::domain::models::{AudioFile, Tag};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainEvent {
    AudioFileAdded(AudioFile),
    AudioFileUpdated(AudioFile),
    TagUpdated(Tag),
}
