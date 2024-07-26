use crate::domain::TagFieldType;
use base64::prelude::*;
use itertools::Itertools;
use regex::Regex;
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
    /// Returns all the tag fields that can be parsed from the given ID3v2 frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The frames to parse the fields from.
    pub fn parse_all(frames: &[ID3v2Frame]) -> Vec<TagField> {
        #[derive(Clone, Debug)]
        enum MultiFrameType {
            Tpos,
            Trck,
        }
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
        enum PartialFrameGroup {
            Date,
        }
        #[derive(Clone, Debug)]
        enum FrameType {
            Multi(MultiFrameType),
            Partial(PartialFrameGroup),
            Single,
        }
        impl From<&ID3v2Frame> for FrameType {
            fn from(frame: &ID3v2Frame) -> Self {
                match frame.header.frame_id.as_str() {
                    "TPOS" => FrameType::Multi(MultiFrameType::Tpos),
                    "TRCK" => FrameType::Multi(MultiFrameType::Trck),
                    "TDAT" => FrameType::Partial(PartialFrameGroup::Date),
                    "TYER" => FrameType::Partial(PartialFrameGroup::Date),
                    _ => FrameType::Single,
                }
            }
        }
        let mut singles = vec![];
        let mut multis = vec![];
        let mut partials = vec![];
        frames
            .iter()
            .for_each(|frame| match FrameType::from(frame) {
                FrameType::Multi(t) => multis.push((t, frame)),
                FrameType::Partial(g) => partials.push((g, frame)),
                FrameType::Single => singles.push(frame),
            });
        singles
            .iter()
            .map(|a| TagField::from(*a))
            .chain(multis.iter().flat_map(|(t, frame)| -> Vec<TagField> {
                match t {
                    MultiFrameType::Tpos => TagField::tpos(frame),
                    MultiFrameType::Trck => TagField::trck(frame),
                }
            }))
            .chain(
                partials
                    .iter()
                    .sorted_by(|(g1, _), (g2, _)| g1.cmp(g2))
                    .chunk_by(|(g, _)| g)
                    .into_iter()
                    .map(|(g, v)| -> TagField {
                        let v: Vec<&ID3v2Frame> = v.map(|(_, f)| *f).collect();
                        match g {
                            PartialFrameGroup::Date => TagField::date(v),
                        }
                    }),
            )
            .collect()
    }

    /// Returns a date tag field for the given frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The frames to parse the date field from (i.e. TYER and TDAT).
    pub fn date(frames: Vec<&ID3v2Frame>) -> TagField {
        let tyer = frames
            .iter()
            .find(|frame| frame.header.frame_id == "TYER")
            .map(|frame| match &frame.fields {
                ID3v2FrameFields::TextFields { encoding: _, text } => String::from(&text[0]),
                _ => todo!(),
            })
            .unwrap_or(String::new());
        let regex = Regex::new("(\\d{2})(\\d{2})").unwrap();
        let mut tdat: Vec<String> = frames
            .iter()
            .find(|frame| frame.header.frame_id == "TDAT")
            .map(|frame| match &frame.fields {
                ID3v2FrameFields::TextFields { encoding: _, text } => regex
                    .captures(&text[0])
                    .map(|c| {
                        c.iter()
                            .skip(1)
                            .filter_map(|c| c.map(|c| c.as_str()).map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                _ => todo!(),
            })
            .unwrap_or_default();
        tdat.push(tyer);
        let s = tdat.join("/");
        TagField::Text(TagFieldType::Date, s, None)
    }

    /// Returns the disc number and total discs fields for the given TPOS frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The TPOS frame to parse the fields from.
    pub fn tpos(frame: &ID3v2Frame) -> Vec<TagField> {
        match &frame.fields {
            ID3v2FrameFields::TextFields { encoding: _, text } => text[0]
                .split('/')
                .map(String::from)
                .enumerate()
                .map(|(i, s)| {
                    (
                        match i {
                            0 => TagFieldType::DiscNumber,
                            _ => TagFieldType::TotalDiscs,
                        },
                        s,
                    )
                })
                .map(|(t, s)| TagField::Text(t, s, None))
                .collect(),
            _ => todo!(),
        }
    }

    /// Returns the track number and total tracks fields for the given TRCK frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The TRCK frame to parse the fields from.
    pub fn trck(frame: &ID3v2Frame) -> Vec<TagField> {
        match &frame.fields {
            ID3v2FrameFields::TextFields { encoding: _, text } => text[0]
                .split('/')
                .map(String::from)
                .enumerate()
                .map(|(i, s)| {
                    (
                        match i {
                            0 => TagFieldType::TrackNumber,
                            _ => TagFieldType::TotalTracks,
                        },
                        s,
                    )
                })
                .map(|(t, s)| TagField::Text(t, s, None))
                .collect(),
            _ => todo!(),
        }
    }

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
                match String::from_utf8(id.clone()) {
                    Ok(s) => TagField::Text(TagFieldType::from(frame), s, None),
                    Err(_) => TagField::Binary(TagFieldType::from(frame), id.clone(), None),
                }
            }
            _ => TagField::Unknown(TagFieldType::from(frame), String::new()),
        }
    }
}
