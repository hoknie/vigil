use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct ContainersCollector {
    absent: NotOnThisSystem,
}

impl ContainersCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ContainersCollector {
            absent: NotOnThisSystem::new(
                "containers",
                format!(
                    "the containers of a host are read from a Linux /proc and its cgroups, and \
                     this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for ContainersCollector {
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
