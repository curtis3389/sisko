use serde::Deserialize;

#[derive(Deserialize)]
pub struct CoverArtArchive {
    pub count: i32,
    pub darkened: bool,
    pub artwork: bool,
    pub front: bool,
    pub back: bool,
}
