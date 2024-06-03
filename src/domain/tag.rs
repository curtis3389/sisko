use crate::domain::TagField;

/// Represents a metadata tag in an audio file.
pub trait Tag {
    /// Returns the artist field, if any.
    fn artist(&self) -> Option<String>;

    /// Returns the name of this tag (e.g. "ID3v2").
    fn tag_name(&self) -> String;

    /// Returns the track title field, if any.
    fn title(&self) -> Option<String>;

    /// Returns the fields in the tag.
    fn fields(&self) -> Vec<TagField>;
}
