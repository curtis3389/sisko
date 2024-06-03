use mime_guess::{self, mime};
use std::path::PathBuf;

/// Represents the possible file types.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub enum FileType {
    /// A directory.
    Directory,
    /// An Audio Interchange File Format file.
    AiffFile,
    /// A Free Lossless Audio Codec file.
    FlacFile,
    /// An MPEG Audio Layer III file.
    Mp3File,
    /// An MPEG-4 Audio file.
    Mp4aFile,
    /// An Ogg Vorbis file.
    OggFile,
    /// A Waveform Audio File Format file.
    WavFile,
    /// A file that is unsupported by this program.
    UnsupportedFile,
}

impl FileType {
    /// Returns the display string for this file type.
    pub fn as_str(&self) -> &str {
        match *self {
            FileType::Directory => "Directory",
            FileType::AiffFile => "AIFF File",
            FileType::FlacFile => "FLAC File",
            FileType::Mp3File => "MP3 File",
            FileType::Mp4aFile => "MP4A File",
            FileType::OggFile => "OGG File",
            FileType::WavFile => "WAV File",
            FileType::UnsupportedFile => "Unsupported File",
        }
    }
}

impl From<&PathBuf> for FileType {
    /// Returns the FileType of the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to get the type of.
    fn from(path: &PathBuf) -> Self {
        if path.is_file() {
            let guess = mime_guess::from_path(path);
            match guess.first() {
                Some(mime) => {
                    if mime.type_() == mime::AUDIO {
                        match mime.subtype().as_str() {
                            "aiff" => FileType::AiffFile,
                            "flac" => FileType::FlacFile,
                            "m4a" => FileType::Mp4aFile,
                            "m4b" => FileType::Mp4aFile,
                            "mp4" => FileType::Mp4aFile,
                            "mpeg" => FileType::Mp3File,
                            "ogg" => FileType::OggFile,
                            "wav" => FileType::WavFile,
                            _ => FileType::UnsupportedFile,
                        }
                    } else {
                        FileType::UnsupportedFile
                    }
                }
                None => FileType::UnsupportedFile,
            }
        } else if path.is_dir() {
            FileType::Directory
        } else {
            FileType::UnsupportedFile
        }
    }
}
