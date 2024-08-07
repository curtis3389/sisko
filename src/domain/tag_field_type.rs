use std::fmt::Display;

use sisko_lib::{id3v2_frame::ID3v2Frame, id3v2_frame_fields::ID3v2FrameFields};

/// Represents the type of a field in a tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TagFieldType {
    AcoustId,
    Album,
    AlbumArtist,
    AlbumArtistSortOrder,
    Artist,
    Artists,
    ArtistSortOrder,
    Asin,
    Barcode,
    CatalogNumber,
    Date,
    DiscNumber,
    DiscSubtitle,
    Isrc,
    Media,
    MusicBrainzArtistId,
    MusicBrainzRecordingId,
    MusicBrainzReleaseArtistId,
    MusicBrainzReleaseGroupId,
    MusicBrainzReleaseId,
    MusicBrainzTrackId,
    OriginalReleaseDate,
    OriginalYear,
    RecordLabel,
    ReleaseCountry,
    ReleaseStatus,
    ReleaseType,
    Script,
    TotalDiscs,
    TotalTracks,
    TrackNumber,
    Title,
    Ufid(String),
    Unknown(String),
}

impl TagFieldType {
    /// Returns the field type for display.
    pub fn display_name(&self) -> String {
        match &self {
            TagFieldType::Album => String::from("Album"),
            TagFieldType::AlbumArtist => String::from("Album Artist"),
            TagFieldType::AlbumArtistSortOrder => String::from("Album Artist Sort Order"),
            TagFieldType::Artist => String::from("Artist"),
            TagFieldType::ArtistSortOrder => String::from("Artist Sort Order"),
            TagFieldType::Asin => String::from("ASIN"),
            TagFieldType::DiscSubtitle => String::from("Disc Subtitle"),
            TagFieldType::Isrc => String::from("ISRC"),
            TagFieldType::Media => String::from("Media"),
            TagFieldType::MusicBrainzRecordingId => String::from("MusicBrainz Recording Id"),
            TagFieldType::OriginalYear => String::from("Original Year"),
            TagFieldType::RecordLabel => String::from("Record Label"),
            TagFieldType::Script => String::from("Script"),
            TagFieldType::Title => String::from("Title"),
            TagFieldType::Ufid(owner_id) => format!("UFID:{}", owner_id),
            TagFieldType::Unknown(id) => format!("Unknown({})", id),
            TagFieldType::AcoustId => String::from("AcoustID"),
            TagFieldType::Artists => String::from("Artists"),
            TagFieldType::Barcode => String::from("Barcode"),
            TagFieldType::CatalogNumber => String::from("Catalog Number"),
            TagFieldType::Date => String::from("Date"),
            TagFieldType::DiscNumber => String::from("Disc Number"),
            TagFieldType::MusicBrainzArtistId => String::from("MusicBrainz Artist Id"),
            TagFieldType::MusicBrainzReleaseArtistId => {
                String::from("MusicBrainz Release Artist Id")
            }
            TagFieldType::MusicBrainzReleaseGroupId => String::from("MusicBrainz Release Group Id"),
            TagFieldType::MusicBrainzReleaseId => String::from("MusicBrainz Release Id"),
            TagFieldType::MusicBrainzTrackId => String::from("MusicBrainz Track Id"),
            TagFieldType::OriginalReleaseDate => String::from("Original Release Date"),
            TagFieldType::ReleaseCountry => String::from("Release Country"),
            TagFieldType::ReleaseStatus => String::from("Release Status"),
            TagFieldType::ReleaseType => String::from("Release Type"),
            TagFieldType::TotalDiscs => String::from("Total Discs"),
            TagFieldType::TotalTracks => String::from("Total Tracks"),
            TagFieldType::TrackNumber => String::from("Track Number"),
        }
    }
}

impl Display for TagFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<&ID3v2Frame> for TagFieldType {
    fn from(frame: &ID3v2Frame) -> Self {
        match frame.header.frame_id.as_str() {
            "TALB" => Self::Album,
            "TIT2" => Self::Title,
            "TMED" => Self::Media,
            "TORY" => Self::OriginalYear,
            "TPE1" => Self::Artist,
            "TPE2" => Self::AlbumArtist,
            "TPUB" => Self::RecordLabel,
            "TSO2" => Self::AlbumArtistSortOrder,
            "TSOP" => Self::ArtistSortOrder,
            "TSRC" => Self::Isrc,
            "TSST" => Self::DiscSubtitle,
            "TXXX" => match &frame.fields {
                ID3v2FrameFields::UserDefinedTextFields {
                    encoding: _,
                    description,
                    value: _,
                } => match description.to_uppercase().as_str() {
                    "ACOUSTID ID" => Self::AcoustId,
                    "ARTISTS" => Self::Artists,
                    "ASIN" => Self::Asin,
                    "BARCODE" => Self::Barcode,
                    "CATALOGNUMBER" => Self::CatalogNumber,
                    "MUSICBRAINZ ALBUM ARTIST ID" => Self::MusicBrainzReleaseArtistId,
                    "MUSICBRAINZ ALBUM ID" => Self::MusicBrainzReleaseId,
                    "MUSICBRAINZ ALBUM STATUS" => Self::ReleaseStatus,
                    "MUSICBRAINZ ALBUM TYPE" => Self::ReleaseType,
                    "MUSICBRAINZ ALBUM RELEASE COUNTRY" => Self::ReleaseCountry,
                    "MUSICBRAINZ ARTIST ID" => Self::MusicBrainzArtistId,
                    "MUSICBRAINZ RELEASE GROUP ID" => Self::MusicBrainzReleaseGroupId,
                    "MUSICBRAINZ RELEASE TRACK ID" => Self::MusicBrainzTrackId,
                    "ORIGINALYEAR" => Self::OriginalReleaseDate,
                    "SCRIPT" => Self::Script,
                    _ => Self::Unknown(description.clone()),
                },
                _ => Self::Unknown(frame.header.frame_id.clone()),
            },
            "UFID" => match &frame.fields {
                ID3v2FrameFields::UniqueFileIdentifierFields { owner_id, id: _ } => {
                    match owner_id.as_str() {
                        "http://musicbrainz.org" => Self::MusicBrainzRecordingId,
                        _ => Self::Ufid(owner_id.clone()),
                    }
                }
                _ => Self::Unknown(frame.header.frame_id.clone()),
            },
            _ => Self::Unknown(frame.header.frame_id.clone()),
        }
    }
}
