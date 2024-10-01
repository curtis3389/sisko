use crate::domain::{
    models::Track,
    repos::{AudioFileRepository, TagRepository},
};
use anyhow::Result;
use std::ops::BitOr;

// TODO: add match confidence to matched variants
#[derive(Clone, Copy, Debug)]
pub enum MatchState {
    MatchedChanges,
    MatchedNoChanges,
    UnmatchedChanges,
    UnmatchedNoChanges,
}

impl MatchState {
    pub async fn for_tracks(tracks: &[Track]) -> Result<Vec<MatchState>> {
        let mut match_states = vec![];
        for track in tracks {
            let matches = AudioFileRepository::instance().get_matched(track).await?;
            let is_matched = !matches.is_empty();
            let mut has_changes: Vec<bool> = vec![];
            for m in matches {
                let tag = TagRepository::instance().get(&m).await?;
                has_changes.push(tag.has_changes());
            }
            let has_changes = has_changes.iter().any(|h| *h);
            match_states.push(MatchState::from((is_matched, has_changes)));
        }
        Ok(match_states)
    }

    pub fn has_changes(&self) -> bool {
        match self {
            MatchState::MatchedChanges | MatchState::UnmatchedChanges => true,
            MatchState::MatchedNoChanges | MatchState::UnmatchedNoChanges => false,
        }
    }

    pub fn is_matched(&self) -> bool {
        match self {
            MatchState::MatchedChanges | MatchState::MatchedNoChanges => true,
            MatchState::UnmatchedChanges | MatchState::UnmatchedNoChanges => false,
        }
    }
}

impl BitOr for MatchState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let is_matched = self.is_matched() && rhs.is_matched();
        let has_changes = self.has_changes() || rhs.has_changes();
        Self::from((is_matched, has_changes))
    }
}

impl From<(bool, bool)> for MatchState {
    fn from((is_matched, has_changes): (bool, bool)) -> Self {
        match (is_matched, has_changes) {
            (true, true) => Self::MatchedChanges,
            (true, false) => Self::MatchedNoChanges,
            (false, true) => Self::UnmatchedChanges,
            (false, false) => Self::UnmatchedNoChanges,
        }
    }
}

impl From<&Vec<MatchState>> for MatchState {
    fn from(match_states: &Vec<MatchState>) -> Self {
        match_states
            .iter()
            .fold(MatchState::MatchedNoChanges, |accumulator, current| {
                accumulator | *current
            })
    }
}
