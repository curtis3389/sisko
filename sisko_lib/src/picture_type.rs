use anyhow::{anyhow, Result};

/// Represents the picture type of an attached picture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PictureType {
    /// An unknown/unspecified type of picture.
    Other,

    /// A 32x32 pixel 'file icon' (PNG only)
    FileIcon,

    /// A general file icon.
    OtherFileIcon,

    /// The front of the cover.
    CoverFront,

    /// The back of the cover.
    CoverBack,

    /// The leaflet page.
    LeafletPage,

    /// The media (e.g. label side of the CD).
    Media,

    /// The lead artist/lead performer/soloist.
    LeadArtist,

    /// The artist/performer.
    Artist,

    /// The conductor.
    Conductor,

    /// The band/orchestra.
    Band,

    /// The composer.
    Composer,

    /// The lyricist/text writer.
    Lyricist,

    /// The recording location.
    RecordingLocation,

    /// A picture from during recording.
    DuringRecording,

    /// A picture from during the performance.
    DuringPerformance,

    /// A movie/video screen capture.
    VideoScreenCapture,

    /// A picture of a brightly coloured fish, duh.
    ABrightColouredFish,

    /// An illustration.
    Illustration,

    /// A band/artist logotype.
    BandLogoType,

    /// A publisher/studio logotype.
    PublisherLogoType,
}

impl PictureType {
    /// Parses the attached picture type from the given byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to parse the picture type from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::picture_type::*;
    /// let pairs = [
    ///     (b'\x00', PictureType::Other),
    ///     (b'\x01', PictureType::FileIcon),
    ///     (b'\x02', PictureType::OtherFileIcon),
    ///     (b'\x03', PictureType::CoverFront),
    ///     (b'\x04', PictureType::CoverBack),
    ///     (b'\x05', PictureType::LeafletPage),
    ///     (b'\x06', PictureType::Media),
    ///     (b'\x07', PictureType::LeadArtist),
    ///     (b'\x08', PictureType::Artist),
    ///     (b'\x09', PictureType::Conductor),
    ///     (b'\x0A', PictureType::Band),
    ///     (b'\x0B', PictureType::Composer),
    ///     (b'\x0C', PictureType::Lyricist),
    ///     (b'\x0D', PictureType::RecordingLocation),
    ///     (b'\x0E', PictureType::DuringRecording),
    ///     (b'\x0F', PictureType::DuringPerformance),
    ///     (b'\x10', PictureType::VideoScreenCapture),
    ///     (b'\x11', PictureType::ABrightColouredFish),
    ///     (b'\x12', PictureType::Illustration),
    ///     (b'\x13', PictureType::BandLogoType),
    ///     (b'\x14', PictureType::PublisherLogoType),
    /// ];
    ///
    /// for (byte, expected) in pairs {
    ///     assert_eq!(PictureType::parse(byte)?, expected);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn parse(byte: u8) -> Result<PictureType> {
        match byte {
            b'\x00' => Ok(PictureType::Other),
            b'\x01' => Ok(PictureType::FileIcon),
            b'\x02' => Ok(PictureType::OtherFileIcon),
            b'\x03' => Ok(PictureType::CoverFront),
            b'\x04' => Ok(PictureType::CoverBack),
            b'\x05' => Ok(PictureType::LeafletPage),
            b'\x06' => Ok(PictureType::Media),
            b'\x07' => Ok(PictureType::LeadArtist),
            b'\x08' => Ok(PictureType::Artist),
            b'\x09' => Ok(PictureType::Conductor),
            b'\x0A' => Ok(PictureType::Band),
            b'\x0B' => Ok(PictureType::Composer),
            b'\x0C' => Ok(PictureType::Lyricist),
            b'\x0D' => Ok(PictureType::RecordingLocation),
            b'\x0E' => Ok(PictureType::DuringRecording),
            b'\x0F' => Ok(PictureType::DuringPerformance),
            b'\x10' => Ok(PictureType::VideoScreenCapture),
            b'\x11' => Ok(PictureType::ABrightColouredFish),
            b'\x12' => Ok(PictureType::Illustration),
            b'\x13' => Ok(PictureType::BandLogoType),
            b'\x14' => Ok(PictureType::PublisherLogoType),
            _ => Err(anyhow!("Unknown picture type: {}", byte)),
        }
    }
}
