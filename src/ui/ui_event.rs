use super::{AlbumView, AudioFileView, TagFieldView};
use crate::domain::{AudioFile, File};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum UiEvent {
    FileSelected(Arc<File>),
    FolderSelected(Arc<File>),
    OpenAddFile,
    OpenAddFolder,
    OpenLogs,
    SaveAudioFile(AudioFile),
    ScanAudioFile(AudioFile),
    SelectAlbumView(AlbumView),
    SelectClusterFile(AudioFileView),
    SubmitAlbumView(AlbumView),
    SubmitClusterFile(AudioFileView),
    SubmitMetadataRow(TagFieldView),
}

impl Display for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
