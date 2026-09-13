#![cfg(target_os = "linux")]

mod files;
mod filesystems;
mod health;
mod reading;
mod source;

#[cfg(test)]
mod tests;

use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health};

use files::Files;
use source::NAME;

pub struct ResourcesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    wall_clock: Box<dyn Fn() -> Option<i64> + Send + Sync>,
    files: Files,
    booted_at: Mutex<Option<i64>>,
}

impl Collector for ResourcesCollector {
    fn name(&self) -> &'static str {
        NAME
    }

    fn available(&self) -> Health {
        self.health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        self.reading()
    }

    fn restore(&self, previous: &Snapshot) {
        self.remember(previous);
    }
}
