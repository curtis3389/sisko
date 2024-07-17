use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextRepresentation {
    pub language: String,
    pub script: String,
}
