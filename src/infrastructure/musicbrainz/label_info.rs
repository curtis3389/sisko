use super::Label;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LabelInfo {
    #[serde(rename = "catalog-number")]
    pub catalog_number: Option<String>,
    pub label: Option<Label>,
}
