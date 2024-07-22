use super::AudioFileColumn;
use crate::domain::{Album, Track};
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
    pub track: Option<Arc<Mutex<Track>>>,
}

impl AlbumView {
    pub fn for_album(album: &Arc<Mutex<Album>>) -> Vec<AlbumView> {
        let mut views = vec![];

        views.push(AlbumView::from(album));
        let am = album.clone();
        let album = album.lock().unwrap();
        let album_id = &album.id;
        let mut tracks: Vec<AlbumView> = album
            .tracks
            .iter()
            .map(|track| AlbumView::for_track(am.clone(), album_id.clone(), track))
            .collect();
        /*tracks.sort_unstable_by(|a, b| {
            let a = {a.album.lock().unwrap().};
            let b = {};
            a.cmp(b)
        });*/
        views.append(&mut tracks);

        views
    }

    pub fn for_track(
        album: Arc<Mutex<Album>>,
        album_id: String,
        track: &Arc<Mutex<Track>>,
    ) -> AlbumView {
        let am = track.clone();
        let track = track.lock().unwrap();
        Self {
            id: track.id.clone(),
            album_id,
            album,
            track: Some(am),
        }
    }

    pub fn artist(&self) -> String {
        match &self.track {
            None => self.album.lock().unwrap().artist.clone(),
            Some(track) => track.lock().unwrap().artist.clone(),
        }
    }

    pub fn disc_number(&self) -> i32 {
        match &self.track {
            None => 0,
            Some(track) => track.lock().unwrap().disc_number,
        }
    }

    pub fn length(&self) -> String {
        match &self.track {
            None => self.album.lock().unwrap().length.clone(),
            Some(track) => track.lock().unwrap().length.clone(),
        }
    }

    pub fn number(&self) -> i32 {
        match &self.track {
            None => 0,
            Some(track) => track.lock().unwrap().number,
        }
    }

    pub fn title(&self) -> String {
        match &self.track {
            None => self.album.lock().unwrap().title.clone(),
            Some(track) => {
                let track = track.lock().unwrap();
                format!("  {}-{} {}", track.disc_number, track.number, track.title)
            }
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
            track: None,
        }
    }
}

impl TableViewItem<AudioFileColumn> for AlbumView {
    /// Returns the value of the given column for this AudioFile.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get the value of.
    fn to_column(&self, column: AudioFileColumn) -> String {
        match column {
            AudioFileColumn::Title => self.title(),
            AudioFileColumn::Artist => self.artist(),
            AudioFileColumn::Length => self.length(),
        }
    }

    /// Compares the value of the given column to another AudioFile.
    ///
    /// # Arguments
    ///
    /// * `other` - The other AudioFile to compare to.
    /// * `column` - The column to compare between the AudioFiles.
    fn cmp(&self, other: &Self, column: AudioFileColumn) -> Ordering
    where
        Self: Sized,
    {
        match self.album_id.cmp(&other.album_id) {
            Ordering::Equal => {
                if self.track.is_none() {
                    Ordering::Less
                } else if other.track.is_none() {
                    Ordering::Greater
                } else {
                    match column {
                        AudioFileColumn::Title => {
                            match self.disc_number().cmp(&other.disc_number()) {
                                Ordering::Equal => match self.number().cmp(&other.number()) {
                                    Ordering::Equal => self.title().cmp(&other.title()),
                                    Ordering::Less => Ordering::Less,
                                    Ordering::Greater => Ordering::Greater,
                                },
                                Ordering::Less => Ordering::Less,
                                Ordering::Greater => Ordering::Greater,
                            }
                        }
                        AudioFileColumn::Artist => self.artist().cmp(&other.artist()),
                        AudioFileColumn::Length => self.length().cmp(&other.length()),
                    }
                }
            }
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}
