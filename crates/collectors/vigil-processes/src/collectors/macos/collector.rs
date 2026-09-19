use vigil_collect::{CollectError, Collector, Health, names_of_users};
use vigil_model::{Rfc3339, Snapshot};

use super::gathering::gathered;
use super::health::health;
use crate::parsers::{ProcessesReading, processes_snapshot_on};
use crate::types::MACOS;

pub struct ProcessesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl ProcessesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ProcessesCollector { now: Box::new(now) }
    }
}

impl Collector for ProcessesCollector {
    fn name(&self) -> &'static str {
        "processes"
    }

    fn available(&self) -> Health {
        health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let gathered = gathered()?;
        let users = names_of_users(gathered.processes.iter().map(|process| process.uid));

        Ok(processes_snapshot_on(
            &(self.now)(),
            &ProcessesReading {
                processes: &gathered.processes,
                users: &users,
                any_unresolved: gathered.unresolved > 0,
            },
            &MACOS,
        ))
    }
}
