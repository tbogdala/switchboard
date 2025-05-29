use sycamore::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SystemMessage(Signal<String>);

impl SystemMessage {
    pub fn new(initial_value: String) -> Self {
        Self(create_signal(initial_value))
    }

    pub fn signal(self) -> Signal<String> {
        self.0
    }
}
