mod health;
mod reading;
mod sockets;
mod source;
mod walk;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::{Rfc3339, Snapshot};

use crate::{CollectError, Collector, Health};

use source::NAME;

pub struct ContainersCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    proc_directory: PathBuf,
    socket_paths: Vec<PathBuf>,
    unread: Mutex<Vec<String>>,
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
