use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};

use super::cron::read_cron;
use super::hooks::read_hooks;
use super::jobs::{homes, read_jobs};
use super::scripts::read_scripts;
use crate::parsers::{MacosPersistenceReading, macos_persistence_snapshot};

pub struct PersistenceCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl PersistenceCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        PersistenceCollector { now: Box::new(now) }
    }
}

impl Collector for PersistenceCollector {
    fn name(&self) -> &'static str {
        "persistence"
    }

    fn available(&self) -> Health {
        super::health::health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let homes = homes();
        let jobs = read_jobs(&homes);
        let (cron, _) = read_cron();
        let mut scripts = read_scripts(&homes);
        scripts.extend(read_hooks().unwrap_or_default());

        if jobs.jobs.is_empty() {
            return Err(CollectError::Denied(format!(
                "not one launchd job could be read, and every Mac has hundreds: {}",
                jobs.unread.join("; ")
            )));
        }

        Ok(macos_persistence_snapshot(
            &(self.now)(),
            &MacosPersistenceReading {
                jobs: &jobs.jobs,
                cron: &cron,
                scripts: &scripts,
            },
        ))
    }
}
