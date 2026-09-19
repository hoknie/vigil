use std::path::{Path, PathBuf};
use std::sync::Mutex;

use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};

use super::seen::Seen;
use crate::spool::{ESLOGGER_SPOOL, ESLOGGER_STATUS, LAUNCHES_DIRECTORY};

pub struct LaunchesCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) spool_path: PathBuf,
    pub(super) status_path: PathBuf,
    pub(super) keep_arguments: bool,
    pub(super) seen: Mutex<Seen>,
}

impl LaunchesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static, keep_arguments: bool) -> Self {
        LaunchesCollector::with_paths(now, keep_arguments, LAUNCHES_DIRECTORY)
    }

    pub fn with_paths(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        keep_arguments: bool,
        directory: impl AsRef<Path>,
    ) -> Self {
        let directory = directory.as_ref();
        LaunchesCollector {
            now: Box::new(now),
            spool_path: directory.join(ESLOGGER_SPOOL),
            status_path: directory.join(ESLOGGER_STATUS),
            keep_arguments,
            seen: Mutex::new(Seen::default()),
        }
    }
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
