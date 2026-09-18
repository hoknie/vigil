#![cfg(target_os = "linux")]

mod filesystems;
mod health;
mod masks;
mod plan;
mod reading;
mod source;
mod stat;
mod walk;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health};

use health::Notes;
use plan::Plan;
use source::NAME;

pub struct FilesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    plan: Plan,
    directories: Vec<PathBuf>,
    last: Mutex<Option<Notes>>,
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
