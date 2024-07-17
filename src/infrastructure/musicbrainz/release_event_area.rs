use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReleaseEventArea {
    // type: Option<>,
    pub id: String,
    pub disambiguation: String,
    #[serde(rename = "iso-3166-1-codes")]
    pub iso_3166_1_codes: Vec<String>,
    // type_id: Option<String>,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    pub name: String,
}
