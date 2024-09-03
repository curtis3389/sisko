use super::{DomainEvent, DomainEventHandler};
use anyhow::Result;
use std::sync::{OnceLock, RwLock};

pub struct MediatorService {
    handlers: RwLock<Vec<DomainEventHandler>>,
}

impl MediatorService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<MediatorService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    pub fn add_handler(&self, handler: DomainEventHandler) {
        let mut handlers = self.handlers.write().unwrap();
        handlers.push(handler);
    }

    pub fn publish(&self, event: &DomainEvent) -> Result<()> {
        let handlers = self.handlers.read().unwrap();
        for handler in handlers.iter() {
            handler(event)?;
        }
        Ok(())
    }
}

impl Default for MediatorService {
    fn default() -> Self {
        Self::new()
    }
}
