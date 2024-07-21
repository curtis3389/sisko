use super::TrackColumn;
use crate::domain::{Album, Recording};
use cursive_table_view::TableViewItem;
use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct AlbumView {
    pub id: String,
    pub album_id: String,
    pub album: Arc<Mutex<Album>>,
    pub recording: Option<Arc<Mutex<Recording>>>,
}

impl AlbumView {
    pub fn for_album(album: &Arc<Mutex<Album>>) -> Vec<AlbumView> {
        let mut views = vec![];

        views.push(AlbumView::from(album));
        let am = album.clone();
        let album = album.lock().unwrap();
        let album_id = &album.id;
        let mut recordings: Vec<AlbumView> = album
            .recordings
            .iter()
            .map(|recording| AlbumView::for_recording(am.clone(), album_id.clone(), recording))
            .collect();
        /*recordings.sort_unstable_by(|a, b| {
            let a = {a.album.lock().unwrap().};
            let b = {};
            a.cmp(b)
        });*/
        views.append(&mut recordings);

        views
    }

    pub fn for_recording(
        album: Arc<Mutex<Album>>,
        album_id: String,
        recording: &Arc<Mutex<Recording>>,
    ) -> AlbumView {
        let am = recording.clone();
        let recording = recording.lock().unwrap();
        Self {
            id: recording.id.clone(),
            album_id,
            album,
            recording: Some(am),
        }
    }

    pub fn artist(&self) -> String {
        match &self.recording {
            None => self.album.lock().unwrap().artist.clone(),
            Some(recording) => recording.lock().unwrap().artist.clone(),
        }
    }

    pub fn length(&self) -> String {
        match &self.recording {
            None => self.album.lock().unwrap().length.clone(),
            Some(recording) => recording.lock().unwrap().length.clone(),
        }
    }

    pub fn title(&self) -> String {
        match &self.recording {
            None => self.album.lock().unwrap().title.clone(),
            Some(recording) => recording.lock().unwrap().title.clone(),
        }
    }
}

impl From<&Arc<Mutex<Album>>> for AlbumView {
    fn from(album: &Arc<Mutex<Album>>) -> Self {
        let am = album.clone();
        let album = album.lock().unwrap();
        Self {
            id: album.id.clone(),
            album_id: album.id.clone(),
            album: am,
            recording: None,
        }
    }
}

impl TableViewItem<TrackColumn> for AlbumView {
    /// Returns the value of the given column for this Track.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: TrackColumn) -> String {
        match column {
            TrackColumn::Title => self.title(),
            TrackColumn::Artist => self.artist(),
            TrackColumn::Length => self.length(),
        }
    }

    /// Compares the value of the given column to another Track.
    ///
    /// # Arguments
    ///
    /// * `other` - The other Track to compare to.
    /// * `column` - The column to compare between the Tracks.
    fn cmp(&self, other: &Self, column: TrackColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            TrackColumn::Title => self.title().cmp(&other.title()),
            TrackColumn::Artist => self.artist().cmp(&other.artist()),
            TrackColumn::Length => self.length().cmp(&other.length()),
        }
    }
}
