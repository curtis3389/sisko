use serde::Deserialize;

#[derive(Deserialize)]
pub struct Label {
    pub disambiguation: String,
    pub id: String,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
    #[serde(rename = "type-id")]
    pub label_type_id: Option<String>,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
}
