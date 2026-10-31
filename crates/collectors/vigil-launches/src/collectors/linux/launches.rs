use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use super::seen::Seen;

use vigil_collect::{CollectError, Collector, Health};

pub struct LaunchesCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) spool_path: PathBuf,
    pub(super) log_path: PathBuf,
    pub(super) plugin_config: PathBuf,
    pub(super) keep_arguments: bool,
    pub(super) seen: Mutex<Seen>,
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
