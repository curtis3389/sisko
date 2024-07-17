use serde::Deserialize;

#[derive(Deserialize)]
pub struct Artist {
    #[serde(rename = "type-id")]
    type_id: String,
    #[serde(rename = "sort-name")]
    sort_name: String,
    name: String,
    //artist_type: String,
    id: String,
    disambiguation: String,
}
