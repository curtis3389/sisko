use super::{AudioFileView, TagFieldView};
use crate::domain::{AudioFile, File};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum UiEvent {
    FileSelected(Arc<File>),
    FolderSelected(Arc<File>),
    OpenAddFile,
    OpenAddFolder,
    OpenLogs,
    ScanAudioFile(Arc<Mutex<AudioFile>>),
    SelectClusterFile(AudioFileView),
    SubmitClusterFile(AudioFileView),
    SubmitMetadataRow(TagFieldView),
}
