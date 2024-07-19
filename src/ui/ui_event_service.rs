use super::{UiEvent, UiEventHandler};
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
                if let Ok(event) = receiver.recv().await {
                    for callback in callbacks.lock().unwrap().iter() {
                        callback(&event);
                    }
                }
            }
        });
        Self { callbacks, sender }
    }

    pub fn send(&self, event: UiEvent) {
        self.sender.send(event).unwrap();
    }

    pub fn subscribe(&self, callback: UiEventHandler) {
        self.callbacks.lock().unwrap().push(callback);
    }
}

impl Default for UiEventService {
    fn default() -> Self {
        Self::new()
    }
}
