use super::AcoustIdRecording;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AcoustIdResult {
    pub id: String,
    pub score: f64,
    pub recordings: Vec<AcoustIdRecording>,
}
