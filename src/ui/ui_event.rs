use super::{TagFieldView, TrackView};
use crate::domain::{File, Track};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum UiEvent {
    FolderSelected(Arc<File>),
    OpenAddFolder,
    OpenLogs,
    ScanTrack(Arc<Mutex<Track>>),
    SelectClusterFile(TrackView),
    SubmitClusterFile(TrackView),
    SubmitMetadataRow(TagFieldView),
}
