use base64::prelude::*;
use rusqlite::{
    types::{Null, ToSqlOutput},
    ToSql,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldValue {
    Binary(Vec<u8>),
    Text(String),
    Unknown,
}

impl FieldValue {
    pub fn discriminator(&self) -> String {
        String::from(match self {
            FieldValue::Binary(_) => "Binary",
            FieldValue::Text(_) => "Text",
            FieldValue::Unknown => "Unknown",
        })
    }
}

pub trait FieldValueExtensions {
    fn display_value(&self) -> String;
}

impl FieldValueExtensions for Option<FieldValue> {
    fn display_value(&self) -> String {
        match self {
            Some(field_value) => match field_value {
                FieldValue::Binary(bytes) => {
                    String::from_utf8(bytes.clone()).unwrap_or(BASE64_STANDARD.encode(bytes))
                }
                FieldValue::Text(text) => text.clone(),
                FieldValue::Unknown => String::from("<unknown>"),
            },
            None => String::new(),
        }
    }
}

impl ToSql for FieldValue {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            FieldValue::Binary(bytes) => bytes.to_sql(),
            FieldValue::Text(text) => text.to_sql(),
            FieldValue::Unknown => Ok(Null.into()),
        }
    }
}

/*
impl TagField {
    /// Returns the type of the field for display.
    pub fn display_name(&self) -> String {
        match &self {
            TagField::Binary(id, _, _) => id.tag_field_type.display_name(),
            TagField::Text(id, _, _) => id.tag_field_type.display_name(),
            TagField::Unknown(id, _) => id.tag_field_type.display_name(),
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

    pub fn has_new_value(&self) -> bool {
        match self {
            TagField::Binary(_, _, new_value) => new_value.is_some(),
            TagField::Text(_, _, new_value) => new_value.is_some(),
            TagField::Unknown(_, _) => false,
        }
    }
}
*/
