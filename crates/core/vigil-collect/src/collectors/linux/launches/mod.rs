mod advice;
mod chunk;
mod health;
mod reading;
mod seen;
mod source;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use seen::Seen;

use crate::{CollectError, Collector, Health};

pub struct LaunchesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    spool_path: PathBuf,
    log_path: PathBuf,
    plugin_config: PathBuf,
    keep_arguments: bool,
    seen: Mutex<Seen>,
}

impl Collector for LaunchesCollector {
    fn name(&self) -> &'static str {
        "launches"
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
