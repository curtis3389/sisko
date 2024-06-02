use cursive::CursiveRunnable;
use std::sync::Mutex;
use syrette::injectable;

/// Represents the default cursive root.
pub trait ICursive {
    /// Gets the default cursive root.
    fn root(&self) -> &Mutex<CursiveRunnable>;
}

/// Represents a wrapper for the default cursive root.
pub struct CursiveWrapper {
    /// The default cursive root.
    root: Mutex<CursiveRunnable>,
}

#[injectable(ICursive)]
impl CursiveWrapper {
    /// Returns a new wrapper for the default cursive root.
    pub fn new() -> Self {
        CursiveWrapper {
            root: Mutex::new(cursive::default()),
        }
    }
}

impl ICursive for CursiveWrapper {
    fn root(&self) -> &Mutex<CursiveRunnable> {
        &self.root
    }
}
