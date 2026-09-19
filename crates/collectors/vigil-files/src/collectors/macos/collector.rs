use std::path::PathBuf;
use std::sync::Mutex;

use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};

use super::health::Notes;
use super::plan::Plan;
use super::source::NAME;

pub struct FilesCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) plan: Plan,
    pub(super) directories: Vec<PathBuf>,
    pub(super) last: Mutex<Option<Notes>>,
}

impl Collector for FilesCollector {
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
