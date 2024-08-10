use crate::{
    domain::TagFieldType,
    infrastructure::{multi_map, MappingType},
};
use anyhow::{anyhow, Result};
use base64::prelude::*;
use itertools::Itertools;
use regex::Regex;
use sisko_lib::{
    id3v2_frame::ID3v2Frame, id3v2_frame_fields::ID3v2FrameFields,
    id3v2_frame_flags::ID3v2FrameFlags,
    id3v2_frame_format_description::ID3v2FrameFormatDescription,
    id3v2_frame_header::ID3v2FrameHeader, id3v2_frame_status_messages::ID3v2FrameStatusMessages,
    text_encoding::TextEncoding,
};

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
    pub fn convert(fields: &[TagField]) -> Vec<ID3v2Frame> {
        #[derive(Clone, Debug)]
        enum MultiFieldType {
            Date,
        }
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
        enum PartialFieldGroup {
            Ipls,
            Tpos,
            Trck,
        }
        impl From<&TagField> for MappingType<MultiFieldType, PartialFieldGroup> {
            fn from(value: &TagField) -> Self {
                match value.tag_field_type() {
                    TagFieldType::Date => MappingType::Multi(MultiFieldType::Date),
                    TagFieldType::DiscNumber => MappingType::Partial(PartialFieldGroup::Tpos),
                    TagFieldType::Engineer => MappingType::Partial(PartialFieldGroup::Ipls),
                    TagFieldType::Mixer => MappingType::Partial(PartialFieldGroup::Ipls),
                    TagFieldType::Performer => MappingType::Partial(PartialFieldGroup::Ipls),
                    TagFieldType::Producer => MappingType::Partial(PartialFieldGroup::Ipls),
                    TagFieldType::TotalDiscs => MappingType::Partial(PartialFieldGroup::Tpos),
                    TagFieldType::TotalTracks => MappingType::Partial(PartialFieldGroup::Trck),
                    TagFieldType::TrackNumber => MappingType::Partial(PartialFieldGroup::Trck),
                    _ => MappingType::Single,
                }
            }
        }

        multi_map(
            fields,
            |f| Ok(ID3v2Frame::from(f)),
            |g, v| -> Result<ID3v2Frame> {
                Ok(match g {
                    PartialFieldGroup::Ipls => TagField::convert_ipls(v),
                    PartialFieldGroup::Tpos => TagField::convert_tpos(v)?,
                    PartialFieldGroup::Trck => TagField::convert_trck(v)?,
                })
            },
            |t, field| -> Result<Vec<ID3v2Frame>> {
                Ok(match t {
                    MultiFieldType::Date => TagField::convert_date(field)?,
                })
            },
        )
        .unwrap()
    }

    pub fn convert_date(field: &TagField) -> Result<Vec<ID3v2Frame>> {
        let date = match field {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let (year, month, day) = date
            .split('-')
            .collect_tuple()
            .ok_or(anyhow!("Failed to parse date field into TDAT and TYER!"))?;
        let year = vec![format!("{:04}", year)];
        let dat = vec![format!("{:02}{:02}", month, day)];
        Ok(vec![
            new_text_frame("TDAT", dat),
            new_text_frame("TYER", year),
        ])
    }

    pub fn convert_ipls(fields: Vec<&TagField>) -> ID3v2Frame {
        let text: Vec<String> = fields
            .iter()
            .flat_map(|field| match field {
                TagField::Text(t, v, n) => n
                    .as_ref()
                    .unwrap_or(v)
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .flat_map(|s| vec![t.to_string(), s]),
                _ => todo!(),
            })
            .collect();
        new_text_frame("IPLS", text)
    }

    pub fn convert_tpos(fields: Vec<&TagField>) -> Result<ID3v2Frame> {
        let disc = fields
            .iter()
            .find(|f| f.tag_field_type() == TagFieldType::DiscNumber)
            .ok_or(anyhow!(
                "Failed to find disc number field to make TPOS frame!"
            ))?;
        let disc = match disc {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let total = fields
            .iter()
            .find(|f| f.tag_field_type() == TagFieldType::TotalDiscs)
            .ok_or(anyhow!(
                "Failed to find total discs field to make TPOS frame!"
            ))?;
        let total = match total {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let text = vec![format!("{}/{}", disc, total)];
        Ok(new_text_frame("TPOS", text))
    }

    pub fn convert_trck(fields: Vec<&TagField>) -> Result<ID3v2Frame> {
        let track = fields
            .iter()
            .find(|f| f.tag_field_type() == TagFieldType::TrackNumber)
            .ok_or(anyhow!(
                "Failed to find track number field to make TRCK frame!"
            ))?;
        let track = match track {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let total = fields
            .iter()
            .find(|f| f.tag_field_type() == TagFieldType::TotalTracks)
            .ok_or(anyhow!(
                "Failed to find total tracks field to make TRCK frame!"
            ))?;
        let total = match total {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let text = vec![format!("{}/{}", track, total)];
        Ok(new_text_frame("TRCK", text))
    }

    /// Returns all the tag fields that can be parsed from the given ID3v2 frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The frames to parse the fields from.
    pub fn parse_all(frames: &[ID3v2Frame]) -> Vec<TagField> {
        #[derive(Clone, Debug)]
        enum MultiFrameType {
            Ipls,
            Tpos,
            Trck,
        }
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
        enum PartialFrameGroup {
            Date,
        }
        impl From<&ID3v2Frame> for MappingType<MultiFrameType, PartialFrameGroup> {
            fn from(frame: &ID3v2Frame) -> Self {
                match frame.header.frame_id.as_str() {
                    "IPLS" => MappingType::Multi(MultiFrameType::Ipls),
                    "TPOS" => MappingType::Multi(MultiFrameType::Tpos),
                    "TRCK" => MappingType::Multi(MultiFrameType::Trck),
                    "TDAT" => MappingType::Partial(PartialFrameGroup::Date),
                    "TYER" => MappingType::Partial(PartialFrameGroup::Date),
                    _ => MappingType::Single,
                }
            }
        }

        multi_map(
            frames,
            |a| Ok(TagField::from(a)),
            |g, v| -> Result<TagField> {
                match g {
                    PartialFrameGroup::Date => TagField::date(v),
                }
            },
            |t, frame| -> Result<Vec<TagField>> {
                Ok(match t {
                    MultiFrameType::Ipls => TagField::ipls(frame),
                    MultiFrameType::Tpos => TagField::tpos(frame),
                    MultiFrameType::Trck => TagField::trck(frame),
                })
            },
        )
        .unwrap()
    }

    /// Returns a date tag field for the given frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The frames to parse the date field from (i.e. TYER and TDAT).
    pub fn date(frames: Vec<&ID3v2Frame>) -> Result<TagField> {
        let tyer = frames
            .iter()
            .find(|frame| frame.header.frame_id == "TYER")
            .and_then(|frame| match &frame.fields {
                ID3v2FrameFields::TextFields { encoding: _, text } => Some(String::from(&text[0])),
                _ => {
                    // TODO: log a warning
                    None
                }
            });
        let regex = Regex::new("(\\d{2})(\\d{2})")?;
        let mut tdat: Vec<String> = frames
            .iter()
            .find(|frame| frame.header.frame_id == "TDAT")
            .and_then(|frame| match &frame.fields {
                ID3v2FrameFields::TextFields { encoding: _, text } => {
                    regex.captures(&text[0]).map(|c| {
                        c.iter()
                            .skip(1)
                            .filter_map(|c| c.map(|c| c.as_str()).map(String::from))
                            .collect()
                    })
                }
                _ => {
                    // TODO: log a warning
                    None
                }
            })
            .unwrap_or_default();
        if let Some(tyer) = tyer {
            tdat.push(tyer);
        }
        let s = tdat.join("/");
        Ok(TagField::Text(TagFieldType::Date, s, None))
    }

    pub fn ipls(frame: &ID3v2Frame) -> Vec<TagField> {
        match &frame.fields {
            ID3v2FrameFields::TextFields { encoding: _, text } => text
                .clone()
                .into_iter()
                .enumerate()
                .chunk_by(|(i, _)| i / 2)
                .into_iter()
                .map(|(_, group)| group.map(|(_, s)| s).collect_vec())
                .map(|g| (g[0].clone(), g[1].clone()))
                .chunk_by(|(a, _)| a.clone())
                .into_iter()
                .map(|(a, group)| (a, group.map(|(_, s)| s).collect_vec()))
                .map(|(a, s)| {
                    TagField::Text(
                        match a.trim().to_lowercase().as_str() {
                            "engineer" => TagFieldType::Engineer,
                            "mix" => TagFieldType::Mixer,
                            "performer" => TagFieldType::Performer,
                            "producer" => TagFieldType::Producer,
                            unknown => TagFieldType::Unknown(format!("IPLS:{}", unknown)),
                        },
                        s.join("; "),
                        None,
                    )
                })
                .collect(),
            _ => todo!(),
        }
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

impl From<&TagField> for ID3v2Frame {
    fn from(field: &TagField) -> Self {
        let text = match field {
            TagField::Text(_, v, n) => n.as_ref().unwrap_or(v).clone(),
            _ => todo!(),
        };
        let text = vec![text];
        match field.tag_field_type() {
            TagFieldType::AcoustId => new_user_text_frame("ACOUSTID ID", text),
            TagFieldType::Album => new_text_frame("TALB", text),
            TagFieldType::AlbumArtist => new_text_frame("TPE2", text),
            TagFieldType::AlbumArtistSortOrder => new_text_frame("TSO2", text),
            TagFieldType::Artist => new_text_frame("TPE1", text),
            TagFieldType::Artists => new_user_text_frame("ARTISTS", text),
            TagFieldType::ArtistSortOrder => new_text_frame("TSOP", text),
            TagFieldType::Asin => new_user_text_frame("ASIN", text),
            TagFieldType::Barcode => new_user_text_frame("BARCODE", text),
            TagFieldType::CatalogNumber => new_user_text_frame("CATALOGNUMBER", text),
            TagFieldType::DiscSubtitle => new_text_frame("TSST", text),
            TagFieldType::Isrc => new_text_frame("TSRC", text),
            TagFieldType::Lyricist => new_text_frame("TEXT", text),
            TagFieldType::Media => new_text_frame("TMED", text),
            TagFieldType::MusicBrainzArtistId => new_user_text_frame("MUSICBRAINZ ARTIST ID", text),
            TagFieldType::MusicBrainzRecordingId => new_ufid_frame("http://musicbrainz.org", text),
            TagFieldType::MusicBrainzReleaseArtistId => {
                new_user_text_frame("MUSICBRAINZ ALBUM ARTIST ID", text)
            }
            TagFieldType::MusicBrainzReleaseGroupId => {
                new_user_text_frame("MUSICBRAINZ RELEASE GROUP ID", text)
            }
            TagFieldType::MusicBrainzReleaseId => new_user_text_frame("MUSICBRAINZ ALBUM ID", text),
            TagFieldType::MusicBrainzTrackId => {
                new_user_text_frame("MUSICBRAINZ RELEASE TRACK ID", text)
            }
            TagFieldType::OriginalReleaseDate => new_user_text_frame("ORIGINALYEAR", text),
            TagFieldType::OriginalYear => new_text_frame("TORY", text),
            TagFieldType::RecordLabel => new_text_frame("TPUB", text),
            TagFieldType::ReleaseCountry => {
                new_user_text_frame("MUSICBRAINZ ALBUM RELEASE COUNTRY", text)
            }
            TagFieldType::ReleaseStatus => new_user_text_frame("MUSICBRAINZ ALBUM STATUS", text),
            TagFieldType::ReleaseType => new_user_text_frame("MUSICBRAINZ ALBUM TYPE", text),
            TagFieldType::Script => new_user_text_frame("SCRIPT", text),
            TagFieldType::Title => new_text_frame("TIT2", text),
            TagFieldType::Ufid(owner_id) => new_ufid_frame(owner_id.as_str(), text),
            TagFieldType::Unknown(_) => todo!(),
            _ => todo!(),
        }
    }
}

fn new_text_frame(id: &str, text: Vec<String>) -> ID3v2Frame {
    let encoding_size: u32 = 1;
    let info_size: u32 = text.iter().map(|s| s.len() as u32 + 1).sum();
    let size = encoding_size + info_size;
    ID3v2Frame {
        header: ID3v2FrameHeader {
            frame_id: String::from(id),
            size,
            flags: ID3v2FrameFlags {
                status_messages: ID3v2FrameStatusMessages {
                    preserve_on_alter_tag: true,
                    preserve_on_alter_file: true,
                    is_read_only: false,
                },
                format_description: ID3v2FrameFormatDescription {
                    is_in_group: false,
                    is_compressed: false,
                    is_encrypted: false,
                    is_unsynchronised: false,
                    has_data_length_indicator: false,
                },
            },
        },
        fields: ID3v2FrameFields::TextFields {
            encoding: TextEncoding::Utf8,
            text,
        },
    }
}

fn new_user_text_frame(description: &str, text: Vec<String>) -> ID3v2Frame {
    let encoding_size: u32 = 1;
    let description_size: u32 = description.len() as u32 + 1;
    let value_size: u32 = text.iter().map(|s| s.len() as u32 + 1).sum();
    let size = encoding_size + description_size + value_size;
    ID3v2Frame {
        header: ID3v2FrameHeader {
            frame_id: String::from("TXXX"),
            size,
            flags: ID3v2FrameFlags {
                status_messages: ID3v2FrameStatusMessages {
                    preserve_on_alter_tag: true,
                    preserve_on_alter_file: true,
                    is_read_only: false,
                },
                format_description: ID3v2FrameFormatDescription {
                    is_in_group: false,
                    is_compressed: false,
                    is_encrypted: false,
                    is_unsynchronised: false,
                    has_data_length_indicator: false,
                },
            },
        },
        fields: ID3v2FrameFields::UserDefinedTextFields {
            encoding: TextEncoding::Utf8,
            description: String::from(description),
            value: text,
        },
    }
}

fn new_ufid_frame(owner_id: &str, text: Vec<String>) -> ID3v2Frame {
    let owner_id = String::from(owner_id);
    let id = text[0].clone();
    let id = id.into_bytes();
    let owner_id_size: u32 = owner_id.len() as u32 + 1;
    let id_size = id.len() as u32;
    let size = owner_id_size + id_size;
    ID3v2Frame {
        header: ID3v2FrameHeader {
            frame_id: String::from("UFID"),
            size,
            flags: ID3v2FrameFlags {
                status_messages: ID3v2FrameStatusMessages {
                    preserve_on_alter_tag: true,
                    preserve_on_alter_file: true,
                    is_read_only: false,
                },
                format_description: ID3v2FrameFormatDescription {
                    is_in_group: false,
                    is_compressed: false,
                    is_encrypted: false,
                    is_unsynchronised: false,
                    has_data_length_indicator: false,
                },
            },
        },
        fields: ID3v2FrameFields::UniqueFileIdentifierFields { owner_id, id },
    }
}
