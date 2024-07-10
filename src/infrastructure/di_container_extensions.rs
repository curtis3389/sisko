use std::{any::type_name, rc::Rc};
use syrette::DIContainer;

/// Represents extension methods to the DIContainer type.
pub trait DIContainerExtensions {
    /// Returns the singleton service of the given type or panics.
    fn expect_singleton<T: 'static + ?Sized>(&self) -> Rc<T>;

    /// Returns the transient service of the given type or panics.
    fn expect_transient<T: 'static + ?Sized>(&self) -> Box<T>;
}

impl DIContainerExtensions for DIContainer {
    fn expect_singleton<T: 'static + ?Sized>(&self) -> Rc<T> {
        self.get::<T>()
            .unwrap_or_else(|_| panic!("Failed to get {} from the DI container!", type_name::<T>()))
            .singleton()
            .unwrap_or_else(|_| panic!("Failed to get {} as a singleton!", type_name::<T>()))
    }

    fn expect_transient<T: 'static + ?Sized>(&self) -> Box<T> {
        self.get::<T>()
            .unwrap_or_else(|_| panic!("Failed to get {} from the DI container!", type_name::<T>()))
            .transient()
            .unwrap_or_else(|_| panic!("Failed to get {} as a transient!", type_name::<T>()))
    }
}
