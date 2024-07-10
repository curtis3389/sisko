use crate::domain::TagFieldType;
use base64::prelude::*;
use sisko_lib::{id3v2_frame::ID3v2Frame, id3v2_frame_fields::ID3v2FrameFields};

/*
TALB = Album
UFID:http://musicbrainz.org = MusicBrainz Recording Id
TORY = Original Year
TMED = Media
TPE2 = Album Artist?
TPUB = Record Label
TSO2 = Album Artist Sort Order?
TSOP = Artist Sort Order?
TSRC = ISRC
TSST = Disc Subtitle
TXXX:SCRIPT = Script
TXXX:ASIN = ASIN

TXXX:originalyear = Original Release Date
TXXX:BARCODE = Barcode
TXXX:ARTISTS = Artists
TXXX:MusicBrainz Album Type = Release Type
TXXX:CATALOGNUMBER = Catalog Number
TXXX:MusicBrainz Album Status = Release Status
TXXX:MusicBrainz Album Release Country = Release Country
TXXX:Acoustid Id = AcoustID
TXXX:MusicBrainz Album Id = MusicBrainz Release Id
TXXX:MusicBrainz Artist Id = MusicBrainz Artist Id
TXXX:MusicBrainz Album Artist Id = MusicBrainz Release Artist Id
TXXX:MusicBrainz Release Group Id = MusicBrainz Release Group Id
TXXX:MusicBrainz Release Track Id = MusicBrainz Track Id


TRCK = Track Number/Total Tracks
TPOS = Disc Number/Total Discs
TYER+TDAT = Date
TDAT = Date of the Recording = Month-day part of Date
TYER = Year part of Date
*/

/// Represents a field in a tag.
#[derive(Clone, Debug)]
pub enum TagField {
    /// A field containing binary data.
    Binary(TagFieldType, Vec<u8>, Option<Vec<u8>>),

    /// A field containing text data.
    Text(TagFieldType, String, Option<String>),

    /// An unrecognized field.
    Unknown(TagFieldType, String),
}

impl TagField {
    /// Returns the type of the field for display.
    pub fn display_name(&self) -> String {
        match &self {
            TagField::Binary(tag_field_type, _, _) => tag_field_type.display_name(),
            TagField::Text(tag_field_type, _, _) => tag_field_type.display_name(),
            TagField::Unknown(tag_field_type, _) => tag_field_type.display_name(),
        }
    }

    /// Returns the value of the field for display.
    pub fn display_value(&self) -> String {
        match &self {
            TagField::Binary(_, value, _) => {
                String::from_utf8(value.clone()).unwrap_or(BASE64_STANDARD.encode(value))
            }
            TagField::Text(_, value, _) => value.clone(),
            TagField::Unknown(_, value) => value.clone(),
        }
    }

    /// Returns the new value of the field for display.
    pub fn display_new_value(&self) -> String {
        match &self {
            TagField::Binary(_, _, new_value) => new_value
                .clone()
                .map(|v| String::from_utf8(v.clone()).unwrap_or(BASE64_STANDARD.encode(v))),
            TagField::Text(_, _, new_value) => new_value.clone(),
            TagField::Unknown(_, _) => None,
        }
        .unwrap_or(String::new())
    }

    /// Returns the type of the field.
    pub fn tag_field_type(&self) -> TagFieldType {
        match &self {
            TagField::Binary(tag_field_type, _, _) => tag_field_type.clone(),
            TagField::Text(tag_field_type, _, _) => tag_field_type.clone(),
            TagField::Unknown(tag_field_type, _) => tag_field_type.clone(),
        }
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
