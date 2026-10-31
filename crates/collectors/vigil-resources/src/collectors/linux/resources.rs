use std::sync::Mutex;
use std::time::Instant;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health};

use super::files::Files;
use super::source::NAME;
use super::storage::Sysfs;

pub struct ResourcesCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) wall_clock: Box<dyn Fn() -> Option<i64> + Send + Sync>,
    pub(super) files: Files,
    pub(super) blocks: Sysfs,
    pub(super) booted_at: Mutex<Option<i64>>,
    pub(super) busy: Mutex<crate::helpers::BusyWindow>,
    pub(super) started: Instant,
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
