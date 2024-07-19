use super::{TagFieldView, TrackView};
use crate::domain::{File, Track};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum UiEvent {
    FileSelected(Arc<File>),
    FolderSelected(Arc<File>),
    OpenAddFile,
    OpenAddFolder,
    OpenLogs,
    ScanTrack(Arc<Mutex<Track>>),
    SelectClusterFile(TrackView),
    SubmitClusterFile(TrackView),
    SubmitMetadataRow(TagFieldView),
}
