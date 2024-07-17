use super::Artist;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArtistCredit {
    name: String,
    joinphrase: String,
    artist: Artist,
}
