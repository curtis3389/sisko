use cursive::CursiveRunnable;
use std::sync::Mutex;

/// Represents the default cursive root.
pub trait ICursive {
    /// Gets the default cursive root.
    fn root(&self) -> &Mutex<CursiveRunnable>;
}
