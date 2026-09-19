use std::sync::Mutex;

use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};

use super::source::{NAME, boot_time};

pub struct ResourcesCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) booted: Box<dyn Fn() -> Option<i64> + Send + Sync>,
    pub(super) booted_at: Mutex<Option<i64>>,
}

impl ResourcesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ResourcesCollector::with_boot_time(now, boot_time)
    }

    pub fn with_boot_time(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        booted: impl Fn() -> Option<i64> + Send + Sync + 'static,
    ) -> Self {
        ResourcesCollector {
            now: Box::new(now),
            booted: Box::new(booted),
            booted_at: Mutex::new(None),
        }
    }
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
