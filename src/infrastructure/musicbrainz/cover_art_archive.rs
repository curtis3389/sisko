use serde::Deserialize;

#[derive(Deserialize)]
pub struct CoverArtArchive {
    count: i32,
    darkened: bool,
    artwork: bool,
    front: bool,
    back: bool,
}
