use crate::id3v2_extended_header::ID3v2ExtendedHeader;
use crate::id3v2_footer::ID3v2Footer;
use crate::id3v2_frame::ID3v2Frame;
use crate::id3v2_header::ID3v2Header;
use anyhow::Result;
use std::fs::File;
use std::io::{prelude::*, SeekFrom};
use std::io::{BufReader, Read};
use std::path::Path;

/// Represents an ID3v2 metadata tag.
#[derive(Clone, Debug)]
pub struct ID3v2Tag {
    /// The header for the tag.
    pub header: ID3v2Header,

    /// (Optional) The extended header for the tag.
    pub extended_header: Option<ID3v2ExtendedHeader>,

    /// The metadata frames in the tag (the actual "tags").
    pub frames: Vec<ID3v2Frame>,

    /// The number of bytes of padding after the frames.
    pub padding: u32,

    /// (Optional) The footer for the tag.
    pub footer: Option<ID3v2Footer>,
}

impl ID3v2Tag {
    /// Parses an ID3v2 tag with the given header from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `header` - The ID3v2 header for the tag.
    /// * `middle_bytes` - The bytes between the header and the footer.
    /// * `footer` - The ID3v2 footer for the tag, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_header::*;
    /// # use sisko_lib::id3v2_header_flags::*;
    /// # use sisko_lib::id3v2_tag::*;
    /// # use sisko_lib::id3v2_version_number::*;
    /// let header = ID3v2Header {
    ///     file_identifier: String::from("ID3"),
    ///     version: ID3v2VersionNumber { major_number: 4, revision_number: 0 },
    ///     flags: ID3v2HeaderFlags {
    ///         unsynchronisation: false,
    ///         has_extended_header: false,
    ///         is_experimental: false,
    ///         has_footer: false,
    ///     },
    ///     size: 15,
    /// };
    /// let bytes = [b'\x54', b'\x50', b'\x4f', b'\x53', b'\x00', b'\x00', b'\x00', b'\x05', b'\x00', b'\x00', b'\x00', b'\x31', b'\x2f', b'\x32', b'\x00'];
    ///
    /// let tag = ID3v2Tag::parse(header, &bytes, None)?;
    ///
    /// assert_eq!(tag.extended_header.is_some(), false);
    /// assert_eq!(tag.frames.len(), 1);
    /// assert_eq!(tag.padding, 0);
    /// assert_eq!(tag.footer.is_some(), false);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn parse(
        header: ID3v2Header,
        middle_bytes: &[u8],
        footer: Option<ID3v2Footer>,
    ) -> Result<ID3v2Tag> {
        let extended_header = match header.flags.has_extended_header {
            true => Some(ID3v2ExtendedHeader::parse(middle_bytes)),
            false => None,
        };
        let frames = ID3v2Frame::parse_all(
            match extended_header {
                None => middle_bytes,
                Some(ref extended_header) => &middle_bytes[(extended_header.size as usize)..],
            },
            &header.version,
        )?;
        let padding = header.size
            - match extended_header {
                None => 0,
                Some(ref extended_header) => extended_header.size,
            }
            - frames.iter().map(|f| f.header.size + 10).sum::<u32>();

        Ok(ID3v2Tag {
            header,
            extended_header,
            frames,
            padding,
            footer,
        })
    }

    /// Reads the ID3v2 tag from the file with the given path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to read the tag from.
    pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<ID3v2Tag> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Reads the ID3v2 tag from the given reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - The stream of bytes to read the tag from.
    pub fn read_from_reader<R: Read + Seek>(reader: &mut R) -> Result<ID3v2Tag> {
        let mut header_bytes: [u8; 10] = [0; 10];
        reader.read_exact(&mut header_bytes)?;
        let header = ID3v2Header::parse(&header_bytes)?;
        let mut middle_bytes: Vec<u8> = vec![0; header.size as usize];
        reader.read_exact(&mut middle_bytes)?;
        let footer = match header.flags.has_footer {
            true => {
                let mut footer_bytes: [u8; 10] = [0; 10];
                reader.read_exact(&mut footer_bytes)?;
                Some(ID3v2Footer::parse(&footer_bytes)?)
            }
            false => None,
        };

        ID3v2Tag::parse(header, &middle_bytes[..], footer)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header_bytes: Vec<u8> = self.header.to_bytes();
        let extended_header_bytes: Vec<u8> = match &self.extended_header {
            Some(extended_header) => extended_header.to_bytes(),
            None => vec![],
        };
        let frame_bytes: Vec<u8> = self
            .frames
            .iter()
            .flat_map(|frame| frame.to_bytes())
            .collect();
        let padding_bytes: Vec<u8> = vec![0u8; self.padding as usize];
        let footer_bytes: Vec<u8> = match &self.footer {
            Some(footer) => footer.to_bytes(),
            None => vec![],
        };

        let mut tag_bytes: Vec<u8> = vec![];
        tag_bytes.extend(header_bytes);
        tag_bytes.extend(extended_header_bytes);
        tag_bytes.extend(frame_bytes);
        tag_bytes.extend(padding_bytes);
        tag_bytes.extend(footer_bytes);
        tag_bytes
    }

    pub fn total_size(&self) -> u32 {
        let header_size = ID3v2Header::total_size();
        let body_size = self.header.size; // extended + frames + padding
        let footer_size = match self.header.flags.has_footer {
            true => ID3v2Footer::total_size(),
            false => 0,
        };
        header_size + body_size + footer_size
    }

    // TODO: remove this method
    pub fn write_to_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // NOTE: new tag size is without padding; padding depends on whether file has header and its size
        // TODO: replace this with ID3v2Tag::has_tag(path)
        let mut file = File::open(path.as_ref())?;
        let file_size = file.seek(SeekFrom::End(0))? as u32;
        file.seek(SeekFrom::Start(0))?;
        match ID3v2Tag::read_from_path(path).ok() {
            Some(tag) => {
                let min_new_tag_size = self.calc_min_size();
                let min_new_size = file_size + min_new_tag_size;
                let padding = (((min_new_size / 2000) + 1) * 2000) - min_new_size;
                self.padding = padding;

                file.seek(SeekFrom::Start(tag.total_size().into()))?;
                let mut file_content: Vec<u8> = vec![];
                file.read_to_end(&mut file_content)?;

                let tag_bytes = self.to_bytes();
                let mut file = File::create_new("tmp.mp3")?;
                file.write_all(&tag_bytes)?;
                file.write_all(file_content.as_slice())?;

                /*if min_new_tag_size <= old_tag_size {
                    // set padding so total size equals old tag
                    // overwrite old tag with new tag
                } else {
                    // then rewrite file with old tag replaced with new
                    // set padding so that size is bigger
                    // read data after tag into memory
                    // write tag and then data to file
                }*/
            }
            None => {
                let padding = 2000;
                self.padding = padding;

                // read file into memory
                let mut reader = BufReader::new(file);
                let mut file_content: Vec<u8> = vec![];
                reader.read_to_end(&mut file_content)?;

                // write tag and then memory to file
                let tag_bytes = self.to_bytes();
                let mut file = File::create_new("tmp.mp3")?;
                file.write_all(&tag_bytes)?;
                file.write_all(file_content.as_slice())?;
            }
        }
        Ok(())
    }

    fn calc_frame_size(&self) -> u32 {
        self.frames.iter().map(|frame| frame.header.size).sum()
    }

    fn calc_min_size(&self) -> u32 {
        let header_size = ID3v2Header::total_size();
        let extended_header_size = match &self.extended_header {
            Some(extended_header) => extended_header.size,
            None => 0,
        };
        let frame_size = self.calc_frame_size();
        let footer_size = ID3v2Footer::total_size();
        header_size + extended_header_size + frame_size + footer_size
    }

    fn calc_padding_to_total(&self, total: u32) -> u32 {
        total - self.calc_min_size()
    }
}
