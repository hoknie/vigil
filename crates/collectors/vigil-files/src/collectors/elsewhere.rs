use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

use crate::types::Listing;

pub struct FilesCollector {
    absent: NotOnThisSystem,
}

impl FilesCollector {
    pub fn new(
        _now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        watched: &[(String, u64)],
    ) -> Self {
        FilesCollector::saying(format!(
            "the {} file(s) this host is configured by are read on Linux and on macOS, and this \
             is {}",
            watched.len(),
            std::env::consts::OS
        ))
    }

    pub fn listed(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static, listing: Listing) -> Self {
        FilesCollector::saying(format!(
            "the paths {} names are read on Linux and on macOS, and this is {}",
            listing.watched_path.display(),
            std::env::consts::OS
        ))
    }

    fn saying(reason: String) -> Self {
        FilesCollector {
            absent: NotOnThisSystem::new("files", reason),
        }
    }
}

impl Collector for FilesCollector {
    fn name(&self) -> &'static str {
        self.absent.name()
    }

    fn available(&self) -> Health {
        self.absent.available()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        self.absent.collect()
    }
}
