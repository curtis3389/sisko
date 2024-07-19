use super::UiEvent;

pub type UiEventHandler = Box<dyn Fn(&UiEvent) + Send>;
