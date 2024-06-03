/// Represents a field in a tag.
#[derive(Clone, Debug)]
pub struct TagField {
    /// The name of the field.
    pub field_name: String,

    /// The original value of the field.
    pub field_value: String,

    /// The new value for the field.
    pub new_field_value: Option<String>,
}
