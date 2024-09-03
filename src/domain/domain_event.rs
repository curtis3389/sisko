use super::AudioFile;

#[derive(Clone, Debug)]
pub enum DomainEvent {
    AudioFileAdded(AudioFile),
    AudioFileUpdated(AudioFile),
}
