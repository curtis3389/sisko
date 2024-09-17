use std::sync::{Arc, Mutex, OnceLock};

pub struct LogHistory {
    memory: Arc<Mutex<Vec<String>>>,
}

impl LogHistory {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<LogHistory> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn logs(&self) -> Arc<Mutex<Vec<String>>> {
        self.memory.clone()
    }
}

impl Default for LogHistory {
    fn default() -> Self {
        Self::new()
    }
}
