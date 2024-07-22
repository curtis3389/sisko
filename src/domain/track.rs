use crate::infrastructure::musicbrainz;

#[derive(Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub length: String,
}

impl From<&musicbrainz::MediaTrack> for Track {
    fn from(r: &musicbrainz::MediaTrack) -> Self {
        let length = r.length.map(|l| l / 1000).unwrap_or_default();
        Self {
            id: r.id.clone(),
            title: r.title.clone(),
            artist: r
                .recording
                .artist_credit
                .first()
                .map(|a| a.artist.name.clone())
                .unwrap_or_default(),
            length: format!("{}:{}", length / 60, length % 60),
        }
    }
}
