use sycamore::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IsSavingChatlog(Signal<bool>);

impl IsSavingChatlog {
    pub fn new(initial_value: bool) -> Self {
        Self(create_signal(initial_value))
    }

    pub fn signal(self) -> Signal<bool> {
        self.0
    }

    pub fn toggle(self) {
        self.0.set(!self.0.get());
    }
}
