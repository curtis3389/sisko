#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TagFieldType {
    AcoustId,
    Album,
    AlbumArtist,
    AlbumArtistSortOrder,
    Artist,
    Artists,
    ArtistSortOrder,
    Asin,
    Barcode,
    CatalogNumber,
    Date,
    DiscNumber,
    DiscSubtitle,
    Isrc,
    Media,
    MusicBrainzArtistId,
    MusicBrainzRecordingId,
    MusicBrainzReleaseArtistId,
    MusicBrainzReleaseGroupId,
    MusicBrainzReleaseId,
    MusicBrainzTrackId,
    OriginalReleaseDate,
    OriginalYear,
    RecordLabel,
    ReleaseCountry,
    ReleaseStatus,
    ReleaseType,
    Script,
    TotalDiscs,
    TotalTracks,
    TrackNumber,
    Title,
    Ufid(String),
    Unknown(String),
}

impl TagFieldType {
    pub fn display_name(&self) -> String {
        match &self {
            TagFieldType::Album => String::from("Album"),
            TagFieldType::AlbumArtist => String::from("Album Artist"),
            TagFieldType::AlbumArtistSortOrder => String::from("Album Artist Sort Order"),
            TagFieldType::Artist => String::from("Artist"),
            TagFieldType::ArtistSortOrder => String::from("Artist Sort Order"),
            TagFieldType::Asin => String::from("ASIN"),
            TagFieldType::DiscSubtitle => String::from("Disc Subtitle"),
            TagFieldType::Isrc => String::from("ISRC"),
            TagFieldType::Media => String::from("Media"),
            TagFieldType::MusicBrainzRecordingId => String::from("MusicBrainz Recording Id"),
            TagFieldType::OriginalYear => String::from("Original Year"),
            TagFieldType::RecordLabel => String::from("Record Label"),
            TagFieldType::Script => String::from("Script"),
            TagFieldType::Title => String::from("Title"),
            TagFieldType::Ufid(owner_id) => format!("UFID:{}", owner_id),
            TagFieldType::Unknown(id) => format!("Unknown({})", id),
            TagFieldType::AcoustId => String::from("AcoustID"),
            TagFieldType::Artists => String::from("Artists"),
            TagFieldType::Barcode => String::from("Barcode"),
            TagFieldType::CatalogNumber => String::from("Catalog Number"),
            TagFieldType::Date => String::from("Date"),
            TagFieldType::DiscNumber => String::from("Disc Number"),
            TagFieldType::MusicBrainzArtistId => String::from("MusicBrainz Artist Id"),
            TagFieldType::MusicBrainzReleaseArtistId => {
                String::from("MusicBrainz Release Artist Id")
            }
            TagFieldType::MusicBrainzReleaseGroupId => String::from("MusicBrainz Release Group Id"),
            TagFieldType::MusicBrainzReleaseId => String::from("MusicBrainz Release Id"),
            TagFieldType::MusicBrainzTrackId => String::from("MusicBrainz Track Id"),
            TagFieldType::OriginalReleaseDate => String::from("Original Release Date"),
            TagFieldType::ReleaseCountry => String::from("Release Country"),
            TagFieldType::ReleaseStatus => String::from("Release Status"),
            TagFieldType::ReleaseType => String::from("Release Type"),
            TagFieldType::TotalDiscs => String::from("Total Discs"),
            TagFieldType::TotalTracks => String::from("Total Tracks"),
            TagFieldType::TrackNumber => String::from("Track Number"),
        }
    }
}
