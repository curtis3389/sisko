use super::{AcoustIdResult, Fingerprint};
use anyhow::Result;
use std::path::Path;

pub trait IAcoustIdService {
    fn get_fingerprint(&self, path: &Path) -> Result<Fingerprint>;
    fn lookup_fingerprint(&self, fingerprint: &Fingerprint) -> Result<Vec<AcoustIdResult>>;
}
