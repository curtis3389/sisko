use super::{AlbumTable, ClusterTable, Menu, MetadataTable};
use std::sync::OnceLock;

pub struct Ui {
    pub album_table: AlbumTable,
    pub cluster_table: ClusterTable,
    pub menu: Menu,
    pub metadata_table: MetadataTable,
}

impl Ui {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<Ui> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            album_table: AlbumTable::new(),
            cluster_table: ClusterTable::new(),
            menu: Menu::new(),
            metadata_table: MetadataTable::new(),
        }
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
