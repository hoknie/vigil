use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health};

use super::source::NAME;

pub struct ContainersCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) proc_directory: PathBuf,
    pub(super) socket_paths: Vec<PathBuf>,
    pub(super) unread: Mutex<Vec<String>>,
}

impl Collector for ContainersCollector {
    fn name(&self) -> &'static str {
        NAME
    }

    fn available(&self) -> Health {
        self.health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        self.reading()
    }
}
