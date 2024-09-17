use super::{AudioFileColumn, MatchState};
use crate::domain::models::{Album, AlbumId, AudioFile, Track, TrackId};
use crate::infrastructure::{DurationExtensions, Entity, EntityId};
use cursive_table_view::TableViewItem;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumViewId {
    Album(AlbumId),
    Track(TrackId),
}

impl AlbumViewId {
    pub fn album_id(&self) -> &AlbumId {
        match self {
            AlbumViewId::Album(album_id) => album_id,
            AlbumViewId::Track(track_id) => &track_id.album_id,
        }
    }
}

impl EntityId for AlbumViewId {
    fn to_string(&self) -> String {
        match self {
            AlbumViewId::Album(album_id) => album_id.to_string(),
            AlbumViewId::Track(track_id) => track_id.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumView {
    pub id: AlbumViewId,
    pub artist: String,
    pub disc_number: i32,
    pub length: String,
    pub number: i32,
    pub title: String,
}

impl AlbumView {
    pub fn all_for_album(
        album: &Album,
        tracks: &[Track],
        match_states: &Vec<MatchState>,
    ) -> Vec<AlbumView> {
        let mut views = vec![];
        views.push(Self::for_album(album, MatchState::from(match_states)));
        let mut tracks: Vec<AlbumView> = tracks
            .iter()
            .enumerate()
            .map(|(index, track)| AlbumView::for_track(track, match_states[index]))
            .collect();
        views.append(&mut tracks);
        views
    }

    pub fn for_album(album: &Album, match_state: MatchState) -> AlbumView {
        let icon = match match_state {
            MatchState::MatchedChanges => "⦿⃰",
            MatchState::MatchedNoChanges => "⦿",
            MatchState::UnmatchedChanges => "⦾⃰",
            MatchState::UnmatchedNoChanges => "⦾",
        };
        Self {
            id: AlbumViewId::Album(album.id.clone()),
            artist: album.artist.clone(),
            disc_number: 0,
            length: album.length.to_pretty_string(),
            number: 0,
            title: format!("{} {}", icon, album.title.clone()),
        }
    }

    pub fn for_track(track: &Track, match_state: MatchState) -> AlbumView {
        let icon = match match_state {
            MatchState::MatchedChanges => "█", // TODO: use match rating to use other block characters: ▁, ▂, ▃, ▄, ▅, ▆, ▇
            MatchState::MatchedNoChanges => "✔",
            MatchState::UnmatchedChanges | MatchState::UnmatchedNoChanges => " ",
        };
        Self {
            id: AlbumViewId::Track(track.id.clone()),
            artist: track.artist.clone(),
            disc_number: track.disc_number,
            length: track.length.to_pretty_string(),
            number: track.number,
            title: format!(
                "  {} {}-{} {}",
                icon, track.disc_number, track.number, track.title
            ),
        }
    }

    pub fn update_for_audio_file(&mut self, _audio_file: &AudioFile) {
        // TODO: idk if there's actually anything to update :shrug:
    }
}

impl Entity for AlbumView {
    type Id = AlbumViewId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: EntityId,
    {
        &self.id
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
            AudioFileColumn::Title => self.title.clone(),
            AudioFileColumn::Artist => self.artist.clone(),
            AudioFileColumn::Length => self.length.clone(),
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
        match (
            &self.id,
            &other.id,
            self.id.album_id() == other.id.album_id(),
        ) {
            // put album row before its track rows
            (AlbumViewId::Album(_), AlbumViewId::Track(_), true) => Ordering::Less,
            (AlbumViewId::Track(_), AlbumViewId::Album(_), true) => Ordering::Greater,

            // sort tracks in same album by their actual column values
            (AlbumViewId::Track(_), AlbumViewId::Track(_), true) => match column {
                AudioFileColumn::Title => match self.disc_number.cmp(&other.disc_number) {
                    Ordering::Equal => match self.number.cmp(&other.number) {
                        Ordering::Equal => self.title.cmp(&other.title),
                        Ordering::Less => Ordering::Less,
                        Ordering::Greater => Ordering::Greater,
                    },
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                },
                AudioFileColumn::Artist => self.artist.cmp(&other.artist),
                AudioFileColumn::Length => self.length.cmp(&other.length),
            },

            // order different album (or equal album rows) by album id
            // TODO: compare by their album rows' display values (i.e. album length/artist/title)
            _ => self.id.album_id().value.cmp(&other.id.album_id().value),
        }
    }
}
