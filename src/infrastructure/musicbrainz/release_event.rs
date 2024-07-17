use super::ReleaseEventArea;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReleaseEvent {
    pub date: String,
    pub area: ReleaseEventArea,
}
