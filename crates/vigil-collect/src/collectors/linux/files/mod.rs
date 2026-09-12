mod health;
mod reading;
mod source;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use vigil_model::{Rfc3339, Snapshot};

use crate::{CollectError, Collector, Health};

use source::NAME;

pub struct FilesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    watched: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    ceiling_bytes: u64,
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
