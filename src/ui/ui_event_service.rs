use super::{UiEvent, UiEventHandler};
use anyhow::{anyhow, Result};
use log::error;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

pub struct UiEventService {
    callbacks: Arc<Mutex<Vec<UiEventHandler>>>,
    sender: Sender<UiEvent>,
}

impl UiEventService {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<UiEventService> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        let (sender, mut receiver) = broadcast::channel::<UiEvent>(16);
        let callbacks = Arc::new(Mutex::new(Vec::<UiEventHandler>::new()));

        let cb = callbacks.clone();
        tokio::spawn(async move {
            let callbacks = cb;
            loop {
                match receiver.recv().await {
                    Ok(event) => match callbacks.lock() {
                        Ok(callbacks) => {
                            for callback in callbacks.iter() {
                                callback(&event);
                            }
                        }
                        Err(e) => {
                            error!("Failed to lock UI event callbacks mutex: {}!", e);
                        }
                    },
                    Err(e) => {
                        error!("Error receiving a UI event from the event channel: {}!", e);
                    }
                }
            }
        });
        Self { callbacks, sender }
    }

    pub fn send(&self, event: UiEvent) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }

    pub fn subscribe(&self, callback: UiEventHandler) -> Result<()> {
        self.callbacks
            .lock()
            .map_err(|_| anyhow!("Error locking callbacks mutex!"))?
            .push(callback);
        Ok(())
    }
}

impl Default for UiEventService {
    fn default() -> Self {
        Self::new()
    }
}
