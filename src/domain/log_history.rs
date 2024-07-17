use crate::domain::ILogHistory;
use std::sync::{Arc, Mutex};
use syrette::injectable;

pub struct LogHistory {
    memory: Arc<Mutex<Vec<String>>>,
}

#[injectable(ILogHistory)]
impl LogHistory {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Default for LogHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ILogHistory for LogHistory {
    fn logs(&self) -> Arc<Mutex<Vec<String>>> {
        self.memory.clone()
    }
}
