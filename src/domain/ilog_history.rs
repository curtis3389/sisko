use std::sync::{Arc, Mutex};

pub trait ILogHistory {
    fn logs(&self) -> Arc<Mutex<Vec<String>>>;
}
