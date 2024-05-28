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
    ///     assert_eq!(PictureType::parse(byte), expected);
    /// }
    /// ```
    pub fn parse(byte: u8) -> PictureType {
        match byte {
            b'\x00' => PictureType::Other,
            b'\x01' => PictureType::FileIcon,
            b'\x02' => PictureType::OtherFileIcon,
            b'\x03' => PictureType::CoverFront,
            b'\x04' => PictureType::CoverBack,
            b'\x05' => PictureType::LeafletPage,
            b'\x06' => PictureType::Media,
            b'\x07' => PictureType::LeadArtist,
            b'\x08' => PictureType::Artist,
            b'\x09' => PictureType::Conductor,
            b'\x0A' => PictureType::Band,
            b'\x0B' => PictureType::Composer,
            b'\x0C' => PictureType::Lyricist,
            b'\x0D' => PictureType::RecordingLocation,
            b'\x0E' => PictureType::DuringRecording,
            b'\x0F' => PictureType::DuringPerformance,
            b'\x10' => PictureType::VideoScreenCapture,
            b'\x11' => PictureType::ABrightColouredFish,
            b'\x12' => PictureType::Illustration,
            b'\x13' => PictureType::BandLogoType,
            b'\x14' => PictureType::PublisherLogoType,
            _ => panic!("Unknown picture type: {}", byte),
        }
    }
}
