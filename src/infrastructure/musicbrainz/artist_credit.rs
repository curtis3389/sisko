use super::Artist;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArtistCredit {
    pub name: String,
    pub joinphrase: String,
    pub artist: Artist,
}
