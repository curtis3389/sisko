use crate::file::File;
use crate::tag_service::ITagService;
use crate::track::Track;
use syrette::injectable;
use syrette::ptr::SingletonPtr;

/// Represents a service for working with tracks (i.e. audio files).
pub trait ITrackService {
    /// Gets the track for the given file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to load as a track.
    fn get(&self, file: &File) -> Track;
}

/// Represents a service for working with tracks.
/// A track is an audio file.
pub struct TrackService {
    /// A service for working with tags.
    tag_service: SingletonPtr<dyn ITagService>,
}

#[injectable(ITrackService)]
impl TrackService {
    /// Returns a new track service.
    ///
    /// # Arguments
    ///
    /// * `tag_service` - A service for working with tags.
    pub fn new(tag_service: SingletonPtr<dyn ITagService>) -> Self {
        TrackService { tag_service }
    }
}

impl ITrackService for TrackService {
    fn get(&self, file: &File) -> Track {
        let tags = self.tag_service.get(file);
        let tag = &tags[0];
        Track {
            title: tag
                .title()
                .map(|s| s.to_string())
                .unwrap_or(file.name.clone()),
            artist: tag.artist().map(|s| s.to_string()).unwrap_or_default(),
            length: "".to_string(),
            file: file.clone(),
            //tag: Rc::from(tag),
        }
    }
}
