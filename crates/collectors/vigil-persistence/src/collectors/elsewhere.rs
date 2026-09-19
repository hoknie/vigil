use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct PersistenceCollector {
    absent: NotOnThisSystem,
}

impl PersistenceCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        PersistenceCollector {
            absent: NotOnThisSystem::new(
                "persistence",
                format!(
                    "what a host starts by itself is read from systemd and cron on Linux and \
                     from launchd and cron on macOS, and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for PersistenceCollector {
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
