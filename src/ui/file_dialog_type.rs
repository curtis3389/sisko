/// Represents the available types of file/folder dialogs.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FileDialogType {
    /// A dialog to select a single audio file.
    AudioFile,
    /// A dialog to select a directory.
    Directory,
}
