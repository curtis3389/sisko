use super::{Recording, Release};
use anyhow::Result;

pub trait IMusicBrainzService {
    fn lookup_recording(&self, recording_id: &str) -> Result<Recording>;
    fn lookup_release(&self, release_id: &str) -> Result<Release>;
}
