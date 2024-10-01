use super::{Album, AudioFile, AudioFileId, FieldValue, TagFieldType, Track};
use crate::infrastructure::{multi_map, MappingType};
use crate::{domain::events::DomainEvent, infrastructure::Entity};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use regex::Regex;
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_frame_flags::ID3v2FrameFlags;
use sisko_lib::id3v2_frame_format_description::ID3v2FrameFormatDescription;
use sisko_lib::id3v2_frame_header::ID3v2FrameHeader;
use sisko_lib::id3v2_frame_status_messages::ID3v2FrameStatusMessages;
use sisko_lib::text_encoding::TextEncoding;
use sisko_lib::{
    id3v2_header::ID3v2Header, id3v2_header_flags::ID3v2HeaderFlags, id3v2_tag::ID3v2Tag,
    id3v2_version_number::ID3v2VersionNumber,
};
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataField {
    pub field_type: TagFieldType,
    pub new_value: Option<FieldValue>,
    pub old_value: Option<FieldValue>,
}

impl MetadataField {
    pub fn value(&self) -> &FieldValue {
        self.new_value.as_ref().or(self.old_value.as_ref()).unwrap()
    }
}

pub struct MetadataIterator<'a> {
    iter: Box<dyn Iterator<Item = &'a MetadataField> + 'a>,
}

impl<'a> Iterator for MetadataIterator<'a> {
    type Item = &'a MetadataField;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub audio_file_id: AudioFileId,

    pub events: Vec<DomainEvent>,

    pub fields: HashMap<TagFieldType, MetadataField>,
}

impl Metadata {
    pub fn new(audio_file_id: AudioFileId, values: HashMap<TagFieldType, FieldValue>) -> Self {
        Self {
            audio_file_id,
            events: vec![],
            fields: values
                .into_iter()
                .map(|(field_type, field_value)| {
                    (
                        field_type.clone(),
                        MetadataField {
                            field_type,
                            new_value: None,
                            old_value: Some(field_value),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn from_fields(audio_file_id: AudioFileId, fields: Vec<MetadataField>) -> Self {
        Self {
            audio_file_id,
            events: vec![],
            fields: fields
                .into_iter()
                .map(|field| (field.field_type.clone(), field))
                .collect(),
        }
    }

    pub fn from_id3v2(audio_file_id: AudioFileId, frames: &[ID3v2Frame]) -> Self {
        // NOTE: unique keeps first occurrence, so higher priority goes first
        let values = Self::parse_all(frames).into_iter().unique().collect();
        Self::new(audio_file_id, values)
    }

    /// Returns the value of the artist field, if there is one.
    pub fn artist(&self) -> Option<String> {
        self.current_value(&TagFieldType::Artist)
            .and_then(|value| match value {
                FieldValue::Text(s) => Some(s.clone()),
                _ => None,
            })
    }

    pub fn current_value(&self, field_type: &TagFieldType) -> Option<&FieldValue> {
        self.new_value(field_type)
            .or_else(|| self.value(field_type))
    }

    pub fn get(&self, field_type: &TagFieldType) -> Option<&MetadataField> {
        self.fields.get(field_type)
    }

    pub fn has_changes(&self) -> bool {
        self.fields
            .iter()
            .any(|(_, field)| field.new_value.is_some())
    }

    pub fn into_id3v2(&self) -> Vec<ID3v2Frame> {
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
        impl From<&MetadataField> for MappingType<MultiFieldType, PartialFieldGroup> {
            fn from(field: &MetadataField) -> Self {
                match field.field_type {
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
            &self.fields.values().cloned().collect_vec(),
            |f| Ok(Self::convert_frame(f)),
            |g, v| -> Result<ID3v2Frame> {
                Ok(match g {
                    PartialFieldGroup::Ipls => Self::convert_ipls(v),
                    PartialFieldGroup::Tpos => Self::convert_tpos(v)?,
                    PartialFieldGroup::Trck => Self::convert_trck(v)?,
                })
            },
            |t, field| -> Result<Vec<ID3v2Frame>> {
                Ok(match t {
                    MultiFieldType::Date => Self::convert_date(field)?,
                })
            },
        )
        .unwrap()
    }

    pub fn iter<'a>(&'a self) -> MetadataIterator<'a> {
        MetadataIterator::<'a> {
            iter: Box::new(self.fields.values()),
        }
    }

    pub fn new_value(&self, field_type: &TagFieldType) -> Option<&FieldValue> {
        self.fields
            .get(field_type)
            .and_then(|field| field.new_value.as_ref())
    }

    pub fn new_value_mut(&mut self, field_type: &TagFieldType) -> Option<&mut FieldValue> {
        self.fields
            .get_mut(field_type)
            .and_then(|field| field.old_value.as_mut())
    }

    /// Returns the value of the track title field, if there is one.
    pub fn title(&self) -> Option<String> {
        self.current_value(&TagFieldType::Title)
            .and_then(|value| match value {
                FieldValue::Text(s) => Some(s.clone()),
                _ => None,
            })
    }

    pub fn update(&mut self, field_type: &TagFieldType, field_value: FieldValue) {
        if let Some(value) = self.new_value_mut(field_type) {
            *value = field_value;
        } else {
            self.fields.insert(
                field_type.clone(),
                MetadataField {
                    field_type: field_type.clone(),
                    new_value: Some(field_value),
                    old_value: None,
                },
            );
        }
    }

    pub fn update_for_match(&mut self, audio_file: &AudioFile, album: &Album, track: &Track) {
        let acoust_id = audio_file.acoust_id.clone();
        self.update(&TagFieldType::Title, FieldValue::Text(track.title.clone()));
        self.update(
            &TagFieldType::Artist,
            FieldValue::Text(track.artist.clone()),
        );
        self.update(&TagFieldType::Album, FieldValue::Text(album.title.clone()));
        self.update(
            &TagFieldType::TrackNumber,
            FieldValue::Text(track.number.to_string()),
        );
        self.update(&TagFieldType::Date, FieldValue::Text(album.date.clone()));
        self.update(
            &TagFieldType::AlbumArtist,
            FieldValue::Text(album.artist.clone()),
        );
        self.update(
            &TagFieldType::AlbumArtistSortOrder,
            FieldValue::Text(album.sort_artist.clone()),
        );
        self.update(
            &TagFieldType::ArtistSortOrder,
            FieldValue::Text(track.sort_artist.clone()),
        );
        self.update(
            &TagFieldType::DiscNumber,
            FieldValue::Text(track.disc_number.to_string()),
        );
        self.update(
            &TagFieldType::DiscSubtitle,
            FieldValue::Text(track.disc_subtitle.clone()),
        );
        self.update(&TagFieldType::Media, FieldValue::Text(track.media.clone()));
        self.update(
            &TagFieldType::MusicBrainzArtistId,
            FieldValue::Text(track.artist_id.clone()),
        );
        self.update(
            &TagFieldType::MusicBrainzRecordingId,
            FieldValue::Text(track.recording_id.clone()),
        );
        self.update(
            &TagFieldType::MusicBrainzReleaseArtistId,
            FieldValue::Text(album.artist_id.clone()),
        );
        self.update(
            &TagFieldType::MusicBrainzReleaseGroupId,
            FieldValue::Text(album.release_group_id.clone()),
        );
        self.update(
            &TagFieldType::MusicBrainzReleaseId,
            FieldValue::Text(album.id.value.clone()),
        );
        self.update(
            &TagFieldType::MusicBrainzTrackId,
            FieldValue::Text(track.id.track_id.clone()),
        );
        self.update(
            &TagFieldType::OriginalReleaseDate,
            FieldValue::Text(track.original_release_date.clone()),
        );
        self.update(
            &TagFieldType::OriginalYear,
            FieldValue::Text(track.original_year.clone()),
        );
        self.update(
            &TagFieldType::ReleaseCountry,
            FieldValue::Text(album.release_country.clone()),
        );
        self.update(
            &TagFieldType::ReleaseStatus,
            FieldValue::Text(album.release_status.clone()),
        );
        self.update(
            &TagFieldType::TotalDiscs,
            FieldValue::Text(album.total_discs.to_string()),
        );
        self.update(
            &TagFieldType::TotalTracks,
            FieldValue::Text(track.total_tracks.to_string()),
        );

        if let Some(acoust_id) = &acoust_id {
            self.update(&TagFieldType::AcoustId, FieldValue::Text(acoust_id.clone()));
        }
        if let Some(asin) = &album.asin {
            self.update(&TagFieldType::Asin, FieldValue::Text(asin.clone()));
        }
        if let Some(barcode) = &album.barcode {
            self.update(&TagFieldType::Barcode, FieldValue::Text(barcode.clone()));
        }
        if let Some(catalog_number) = &album.catalog_number {
            self.update(
                &TagFieldType::CatalogNumber,
                FieldValue::Text(catalog_number.clone()),
            );
        }
        if let Some(isrc) = &track.isrc {
            self.update(&TagFieldType::Isrc, FieldValue::Text(isrc.clone()));
        }

        if let Some(record_label) = &album.record_label {
            self.update(
                &TagFieldType::RecordLabel,
                FieldValue::Text(record_label.clone()),
            );
        }
        if let Some(release_type) = &album.release_type {
            self.update(
                &TagFieldType::ReleaseType,
                FieldValue::Text(release_type.clone()),
            );
        }
        if let Some(script) = &album.script {
            self.update(&TagFieldType::Script, FieldValue::Text(script.clone()));
        }
    }

    pub fn value(&self, field_type: &TagFieldType) -> Option<&FieldValue> {
        self.fields
            .get(field_type)
            .and_then(|field| field.old_value.as_ref())
    }

    fn convert_date(field: &MetadataField) -> Result<Vec<ID3v2Frame>> {
        let date = match field.value() {
            FieldValue::Text(s) => s.clone(),
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

    fn convert_frame(field: &MetadataField) -> ID3v2Frame {
        let text = match field.value() {
            FieldValue::Text(s) => s.clone(),
            _ => todo!(),
        };
        let text = vec![text];
        match &field.field_type {
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

    fn convert_ipls(fields: Vec<&MetadataField>) -> ID3v2Frame {
        let text: Vec<String> = fields
            .iter()
            .flat_map(|field| match field.value() {
                FieldValue::Text(s) => s
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .flat_map(|s| vec![field.field_type.to_string(), s]),
                _ => todo!(),
            })
            .collect();
        new_text_frame("IPLS", text)
    }

    fn convert_tpos(fields: Vec<&MetadataField>) -> Result<ID3v2Frame> {
        let disc = fields
            .iter()
            .find(|field| field.field_type == TagFieldType::DiscNumber)
            .ok_or(anyhow!(
                "Failed to find disc number field to make TPOS frame!"
            ))?;
        let disc = match disc.value() {
            FieldValue::Text(s) => s.clone(),
            _ => todo!(),
        };
        let total = fields
            .iter()
            .find(|field| field.field_type == TagFieldType::TotalDiscs)
            .ok_or(anyhow!(
                "Failed to find total discs field to make TPOS frame!"
            ))?;
        let total = match total.value() {
            FieldValue::Text(s) => s.clone(),
            _ => todo!(),
        };
        let text = vec![format!("{}/{}", disc, total)];
        Ok(new_text_frame("TPOS", text))
    }

    fn convert_trck(fields: Vec<&MetadataField>) -> Result<ID3v2Frame> {
        let track = fields
            .iter()
            .find(|field| field.field_type == TagFieldType::TrackNumber)
            .ok_or(anyhow!(
                "Failed to find track number field to make TRCK frame!"
            ))?;
        let track = match track.value() {
            FieldValue::Text(s) => s.clone(),
            _ => todo!(),
        };
        let total = fields
            .iter()
            .find(|field| field.field_type == TagFieldType::TotalTracks)
            .ok_or(anyhow!(
                "Failed to find total tracks field to make TRCK frame!"
            ))?;
        let total = match total.value() {
            FieldValue::Text(s) => s.clone(),
            _ => todo!(),
        };
        let text = vec![format!("{}/{}", track, total)];
        Ok(new_text_frame("TRCK", text))
    }

    fn parse_all(frames: &[ID3v2Frame]) -> Vec<(TagFieldType, FieldValue)> {
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
            |a| Ok(Self::parse_frame(a)),
            |g, v| -> Result<(TagFieldType, FieldValue)> {
                match g {
                    PartialFrameGroup::Date => Self::parse_date(v),
                }
            },
            |t, frame| -> Result<Vec<(TagFieldType, FieldValue)>> {
                Ok(match t {
                    MultiFrameType::Ipls => Self::parse_ipls(frame),
                    MultiFrameType::Tpos => Self::parse_tpos(frame),
                    MultiFrameType::Trck => Self::parse_trck(frame),
                })
            },
        )
        .unwrap()
    }

    fn parse_frame(frame: &ID3v2Frame) -> (TagFieldType, FieldValue) {
        match &frame.fields {
            ID3v2FrameFields::TextFields { encoding: _, text } => {
                (TagFieldType::from(frame), FieldValue::Text(text[0].clone()))
            }
            ID3v2FrameFields::UserDefinedTextFields {
                encoding: _,
                description: _,
                value,
            } => (
                TagFieldType::from(frame),
                FieldValue::Text(value[0].clone()),
            ),
            ID3v2FrameFields::UniqueFileIdentifierFields { owner_id: _, id } => {
                match String::from_utf8(id.clone()) {
                    Ok(s) => (TagFieldType::from(frame), FieldValue::Text(s)),
                    Err(_) => (TagFieldType::from(frame), FieldValue::Binary(id.clone())),
                }
            }
            _ => (TagFieldType::from(frame), FieldValue::Unknown),
        }
    }

    fn parse_date(frames: Vec<&ID3v2Frame>) -> Result<(TagFieldType, FieldValue)> {
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
        Ok((TagFieldType::Date, FieldValue::Text(s)))
    }

    fn parse_ipls(frame: &ID3v2Frame) -> Vec<(TagFieldType, FieldValue)> {
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
                    let field_type = match a.trim().to_lowercase().as_str() {
                        "engineer" => TagFieldType::Engineer,
                        "mix" => TagFieldType::Mixer,
                        "performer" => TagFieldType::Performer,
                        "producer" => TagFieldType::Producer,
                        unknown => TagFieldType::Unknown(format!("IPLS:{}", unknown)),
                    };
                    (field_type, FieldValue::Text(s.join("; ")))
                })
                .collect(),
            _ => todo!(),
        }
    }

    fn parse_tpos(frame: &ID3v2Frame) -> Vec<(TagFieldType, FieldValue)> {
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
                        FieldValue::Text(s),
                    )
                })
                .collect(),
            _ => todo!(),
        }
    }

    fn parse_trck(frame: &ID3v2Frame) -> Vec<(TagFieldType, FieldValue)> {
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
                        FieldValue::Text(s),
                    )
                })
                .collect(),
            _ => todo!(),
        }
    }
}

impl Entity for Metadata {
    type Id = AudioFileId;

    fn id(&self) -> &Self::Id
    where
        Self::Id: crate::infrastructure::EntityId,
    {
        &self.audio_file_id
    }
}

impl Eq for Metadata {}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.audio_file_id == other.audio_file_id && self.fields == other.fields
    }
}

impl From<&Metadata> for ID3v2Tag {
    fn from(value: &Metadata) -> Self {
        let frames = value.into_id3v2();
        let header = ID3v2Header {
            file_identifier: String::from("ID3"),
            version: ID3v2VersionNumber::new(4, 0),
            flags: ID3v2HeaderFlags {
                unsynchronisation: false,
                has_extended_header: false,
                is_experimental: false,
                has_footer: false,
            },
            size: frames.iter().map(|frame| frame.header.size).sum(),
        };
        ID3v2Tag {
            header,
            // TODO: save extended header with restriction settings
            extended_header: None,
            frames,
            // padding and footer determined when saving
            padding: 0,
            footer: None,
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
