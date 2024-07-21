use serde::Deserialize;

#[derive(Deserialize)]
pub struct Artist {
    #[serde(rename = "type-id")]
    pub type_id: Option<String>,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    pub name: String,
    //artist_type: String,
    pub id: String,
    pub disambiguation: String,
}
