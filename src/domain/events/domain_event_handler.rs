use super::DomainEvent;
use anyhow::Result;

pub type DomainEventHandler = Box<dyn Fn(&DomainEvent) -> Result<()> + Send + Sync>;
