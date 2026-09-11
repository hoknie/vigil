use std::sync::{Arc, Mutex};

use super::State;

#[derive(Clone)]
pub struct Shared(Arc<Mutex<State>>);

impl Shared {
    pub fn new(state: State) -> Self {
        Shared(Arc::new(Mutex::new(state)))
    }

    pub fn with<R>(&self, act: impl FnOnce(&mut State) -> R) -> R {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        act(&mut state)
    }
}
