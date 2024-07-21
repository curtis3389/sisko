use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextRepresentation {
    pub language: Option<String>,
    pub script: Option<String>,
}
