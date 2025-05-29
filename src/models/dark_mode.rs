use sycamore::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DarkMode(Signal<bool>);

impl DarkMode {
    pub fn new(initial_value: bool) -> Self {
        Self(create_signal(initial_value))
    }

    pub fn is_dark_mode(self) -> bool {
        self.0.get()
    }

    pub fn signal(self) -> Signal<bool> {
        self.0
    }

    pub fn toggle(self) {
        self.0.set(!self.0.get());
    }
}
