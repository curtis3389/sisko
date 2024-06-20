use crate::domain::{File, Tag, TagField, TagFieldType, TagType};
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use syrette::injectable;

/// Represents a service for working with tags.
pub trait ITagService {
    fn get_all(&self, file: &File) -> Vec<Tag>;
}

/// Represents a service for working with audio file tags.
pub struct TagService {}

#[injectable(ITagService)]
impl TagService {
    /// Returns a new tag service.
    pub fn new() -> Self {
        TagService {}
    }
}

impl ITagService for TagService {
    fn get_all(&self, file: &File) -> Vec<Tag> {
        let id3v2 = ID3v2Tag::read_from_path(&file.path).unwrap();
        let id3v2 = Tag::from(&id3v2);
        vec![id3v2]
    }
}

impl From<&ID3v2Tag> for Tag {
    fn from(id3v2: &ID3v2Tag) -> Self {
        let fields = id3v2
            .frames
            .iter()
            .map(|frame| TagField::from(frame))
            .collect();
        Tag::new(TagType::ID3v2, fields)
    }
}

impl From<&ID3v2Frame> for TagField {
    fn from(frame: &ID3v2Frame) -> Self {
        match &frame.fields {
            ID3v2FrameFields::TextFields { encoding: _, text } => {
                TagField::Text(TagFieldType::from(frame), text[0].clone(), None)
            }
            ID3v2FrameFields::UserDefinedTextFields {
                encoding: _,
                description: _,
                value,
            } => TagField::Text(TagFieldType::from(frame), value[0].clone(), None),
            ID3v2FrameFields::UniqueFileIdentifierFields { owner_id: _, id } => {
                TagField::Binary(TagFieldType::from(frame), id.clone(), None)
            }
            _ => TagField::Unknown(TagFieldType::from(frame), String::new()),
        }
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
