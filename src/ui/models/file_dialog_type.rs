use crate::infrastructure::Value;

/// Represents the available types of file/folder dialogs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileDialogType {
    /// A dialog to select a single audio file.
    AudioFile,
    /// A dialog to select a directory.
    Directory,
}

impl Value for FileDialogType {}
