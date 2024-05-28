use crate::id3v2_frame_header::ID3v2FrameHeader;
use crate::picture_type::PictureType;
use crate::text_encoding::TextEncoding;
use std::fmt::Debug;

/// Represents the possible sets of fields an ID3v2 frame can have.
#[derive(Clone, Debug)]
pub enum ID3v2FrameFields {
    /// A picture directly related to the audio file.
    AttachedPictureFields {
        /// The encoding of the description text.
        encoding: TextEncoding,

        /// The MIME type for the picture.
        mime_type: String,

        /// The type of this picture.
        picture_type: PictureType,

        /// The description of the picture.
        description: String,

        /// The binary data of the picture.
        picture_data: Vec<u8>,
    },

    /// Audio encryption fields if the audio stream is encrypted.
    AudioEncryptionFields {},

    /// A list of seek points within the audio file.
    AudioPointSeekIndexFields {},

    /// Any kind of full text information that does not fit in any other frame.
    CommentsFields {
        /// The encoding of the description and text.
        encoding: TextEncoding,

        /// The language of the text as a 3-character string.
        language: String,

        /// The descritpion of the comment.
        description: String,

        /// The actual text of the comment.
        text: Vec<String>,
    },

    /// Enables several competing offers (i.e. ads).
    CommercialFields {},

    /// Identifies which method a frame has been encrypted with.
    EncryptionFields {},

    /// A pre-defined equalisation curve.
    EqualisationFields {},

    /// Timing codes to allow synchronisation with key events in the audio.
    EventTimingCodesFields {},

    /// Any type of file encapsulated in the tag.
    GeneralObjectFields {},

    /// A group for grouping unrelated frames.
    GroupRegistrationFields {},

    /// A frame linked into this tag from a tag in another file.
    /// TODO: poke other parses for security vulnerabilities here
    /// TODO: try infinite loop frame link
    LinkedInfoFields {},

    /// References that software can use to calculate positions in the file
    /// to increase performance and accuracy of jumps within an MPEG audio file.
    MpegLocationLookupFields {},

    /// A binary dump of the Table of Contents for the CD for identification in
    /// a DB such as CDDB.
    MusicCdIdentifierFields {},

    /// A reminder of a made transaction or, if signed, proof.
    OwnershipFields {},

    /// A counter of the number of times a file has been played.
    PlayCounterFields {},

    /// Specifies how good an audio file is.
    PopularimeterFields {},

    /// Delivers information to the listener of how far into the audio stream
    /// they have picked up.
    PositionSyncFields {},

    /// Contains inforamtion from a software producer that its program uses that
    /// doesn't fit into the other frames.
    PrivateFields {},

    /// The size of the buffer recommended by the server.
    RecommendedBufferFields {},

    /// Allows the user to adjust the volume on each channel.
    RelativeVolumeAdjustmentFields {},

    /// Allows the user to adjust echoes of different kinds.
    ReverbFields {},

    /// Indicates where other tags in the file/stream can be found.
    SeekFields {},

    /// Enables a group of frames to be signed.
    SignatureFields {},

    /// The lyrics of the song on a text transcription of other vocal activities
    /// synchronised with the audio.
    SynchronisedLyricsFields {},

    /// A more accurate descrption of the tempo of a musical piece.
    SynchronisedTempCodesFields {},

    /// A brief description of the terms of use and ownership of the file.
    TermsOfUseFields {},

    /// Textual information about the file like artist, album, and more.
    TextFields {
        /// The encoding of the field.
        encoding: TextEncoding,

        /// The text of the field.
        text: Vec<String>,
    },

    /// A database identifier for the audio file.
    UniqueFileIdentifierFields {
        /// An email or link to find an email for the organisation responsible
        /// for the database that `id` is from.
        owner_id: String,

        /// The unique file ID for this file.
        id: Vec<u8>,
    },

    /// An unrecognized frame's data.
    UnknownFrameFields {
        /// The bytes of the unrecognized frame's fields.
        bytes: Vec<u8>,
    },

    /// The lyrics of the song on a text transcription of other vocal activities.
    UnsynchronisedLyricsFields {},

    /// A URL concerning the audio file.
    UrlFields {},

    /// A user-defined, one-string text information concerning the audio file.
    UserDefinedTextFields {
        /// The encoding of the field.
        encoding: TextEncoding,

        /// The description of the field.
        description: String,

        /// The text of the field.
        value: Vec<String>,
    },

    /// A user-defined URL links concerning the audio file.
    UserDefinedUrlFields {},
}

impl ID3v2FrameFields {
    /// Parses an ID3v2 frame field for the given header from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `header` - The header for the frame to parse the fields for.
    /// * `bytes` - The bytes to parse the fields from.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_frame_fields::*;
    /// # use sisko_lib::id3v2_frame_flags::*;
    /// # use sisko_lib::id3v2_frame_header::*;
    /// # use sisko_lib::text_encoding::*;
    /// let header = ID3v2FrameHeader {
    ///     frame_id: String::from("TPOS"),
    ///     size: 5,
    ///     flags: ID3v2FrameFlags::parse(&[0, 0]),
    /// };
    /// let bytes = [b'\x00', b'\x31', b'\x2f', b'\x32', b'\x00'];
    ///
    /// let fields = ID3v2FrameFields::parse(&header, &bytes);
    ///
    /// if let ID3v2FrameFields::TextFields { encoding, text } = fields {
    ///     assert_eq!(encoding, TextEncoding::Iso88591);
    ///     assert_eq!(text.len(), 1);
    ///     assert_eq!(text[0], "1/2");
    /// } else { panic!(); }
    /// ```
    pub fn parse(header: &ID3v2FrameHeader, bytes: &[u8]) -> ID3v2FrameFields {
        let frame_id = header.frame_id.as_str();
        let fields: ID3v2FrameFields = if frame_id.starts_with('T') && frame_id != "TXXX" {
            let encoding = TextEncoding::parse(bytes[0]);
            let text = encoding.decode(&bytes[1..]);
            ID3v2FrameFields::TextFields { encoding, text }
        } else if frame_id.starts_with('W') && frame_id != "WXXX" {
            ID3v2FrameFields::UrlFields {}
        } else {
            match frame_id {
                "AENC" => ID3v2FrameFields::AudioEncryptionFields {},
                "APIC" => {
                    let encoding = TextEncoding::parse(bytes[0]);
                    let index = 1 + TextEncoding::Iso88591.next_terminator(&bytes[1..]).unwrap();
                    let mime_type = TextEncoding::Iso88591.decode(&bytes[1..index]).remove(0);
                    let picture_type = PictureType::parse(bytes[index + 1]);
                    let next_index = index
                        + 3
                        + encoding.next_terminator(&bytes[index + 2..]).unwrap()
                        + encoding.terminator_width();
                    let description = encoding.decode(&bytes[index + 2..next_index]).remove(0);
                    let picture_data = bytes[next_index..].to_vec();

                    ID3v2FrameFields::AttachedPictureFields {
                        encoding,
                        mime_type,
                        picture_type,
                        description,
                        picture_data,
                    }
                }
                "ASPI" => ID3v2FrameFields::AudioPointSeekIndexFields {},
                "COMM" => {
                    let encoding = TextEncoding::parse(bytes[0]);
                    let language = TextEncoding::Iso88591.decode(&bytes[1..4]).remove(0);
                    let mut text = encoding.decode(&bytes[4..]);
                    let description = text.remove(0);

                    ID3v2FrameFields::CommentsFields {
                        encoding,
                        language,
                        description,
                        text,
                    }
                }
                "COMR" => ID3v2FrameFields::CommercialFields {},
                "ENCR" => ID3v2FrameFields::EncryptionFields {},
                "EQU2" => ID3v2FrameFields::EqualisationFields {},
                "ETCO" => ID3v2FrameFields::EventTimingCodesFields {},
                "GEOB" => ID3v2FrameFields::GeneralObjectFields {},
                "GRID" => ID3v2FrameFields::GroupRegistrationFields {},
                "LINK" => ID3v2FrameFields::LinkedInfoFields {},
                "MCDI" => ID3v2FrameFields::MusicCdIdentifierFields {},
                "MLLT" => ID3v2FrameFields::MpegLocationLookupFields {},
                "OWNE" => ID3v2FrameFields::OwnershipFields {},
                "PCNT" => ID3v2FrameFields::PlayCounterFields {},
                "POPM" => ID3v2FrameFields::PopularimeterFields {},
                "POSS" => ID3v2FrameFields::PositionSyncFields {},
                "PRIV" => ID3v2FrameFields::PrivateFields {},
                "RBUF" => ID3v2FrameFields::RecommendedBufferFields {},
                "RVA2" => ID3v2FrameFields::RelativeVolumeAdjustmentFields {},
                "RVRB" => ID3v2FrameFields::ReverbFields {},
                "SEEK" => ID3v2FrameFields::SeekFields {},
                "SIGN" => ID3v2FrameFields::SignatureFields {},
                "SYLT" => ID3v2FrameFields::SynchronisedLyricsFields {},
                "SYTC" => ID3v2FrameFields::SynchronisedTempCodesFields {},
                "TXXX" => {
                    let encoding = TextEncoding::parse(bytes[0]);
                    let mut value = encoding.decode(&bytes[1..]);
                    let description = value.remove(0);
                    ID3v2FrameFields::UserDefinedTextFields {
                        encoding,
                        description,
                        value,
                    }
                }
                "UFID" => {
                    let (index, _) = bytes
                        .iter()
                        .enumerate()
                        .filter(|(_index, &byte)| byte == 0)
                        .take(1)
                        .collect::<Vec<(usize, &u8)>>()[0];
                    ID3v2FrameFields::UniqueFileIdentifierFields {
                        owner_id: TextEncoding::Iso88591.decode(&bytes[..index]).remove(0),
                        id: bytes[(index + 1)..].to_vec(),
                    }
                }
                "USER" => ID3v2FrameFields::TermsOfUseFields {},
                "USLT" => ID3v2FrameFields::UnsynchronisedLyricsFields {},
                "WXXX" => ID3v2FrameFields::UserDefinedUrlFields {},
                _ => ID3v2FrameFields::UnknownFrameFields {
                    bytes: bytes.to_vec(),
                },
            }
        };
        fields
    }
}
