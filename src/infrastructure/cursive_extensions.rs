use cursive::Cursive;
use syrette::DIContainer;

/// Represents extension methods to the Cursive type.
pub trait CursiveExtensions {
    /// Returns the DIContainer stored in user data.
    fn di_container(&mut self) -> &mut DIContainer;
}

impl CursiveExtensions for Cursive {
    fn di_container(&mut self) -> &mut DIContainer {
        self.user_data()
            .expect("Failed to get DI container from cursive user data!")
            as &mut DIContainer
    }
}
