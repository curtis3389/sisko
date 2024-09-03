use super::Album;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub struct AlbumRepository {
    albums: RwLock<HashMap<String, Album>>,
}

impl AlbumRepository {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<AlbumRepository> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            albums: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, album: Album) {
        self.albums.write().unwrap().insert(album.id.clone(), album);
    }

    pub fn get(&self, id: &String) -> Option<Album> {
        self.albums.read().unwrap().get(id).cloned()
    }

    pub fn remove(&self, album: &Album) {
        self.albums.write().unwrap().remove(&album.id);
    }

    pub fn save(&self, album: Album) {
        self.add(album);
    }
}

impl Default for AlbumRepository {
    fn default() -> Self {
        Self::new()
    }
}
