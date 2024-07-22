pub mod acoustid;
pub mod musicbrainz;

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub type Am<T> = Arc<Mutex<T>>;
pub type Ram<T> = Result<Am<T>>;
