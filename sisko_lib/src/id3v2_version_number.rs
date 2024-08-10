/// Represents the version number of an ID3v2 tag.
#[derive(Clone, Debug)]
pub struct ID3v2VersionNumber {
    /// The major version number of the tag.
    pub major_number: u8,

    /// The revision number of the tag.
    pub revision_number: u8,
}

impl ID3v2VersionNumber {
    /// Returns a new version number for an ID3v2 tag from the given bytes.
    ///
    /// # Arguments
    ///
    /// * `major_number` - The major version number.
    /// * `revision_number` - The revision number.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::id3v2_version_number::*;
    /// let bytes = [b'\x04', b'\x00'];
    ///
    /// let version = ID3v2VersionNumber::new(bytes[0], bytes[1]);
    ///
    /// assert_eq!(version.major_number, 4);
    /// assert_eq!(version.revision_number, 0);
    /// ```
    pub fn new(major_number: u8, revision_number: u8) -> ID3v2VersionNumber {
        ID3v2VersionNumber {
            major_number,
            revision_number,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.major_number, self.revision_number]
    }
}
