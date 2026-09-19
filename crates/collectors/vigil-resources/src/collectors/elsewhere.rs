use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct ResourcesCollector {
    absent: NotOnThisSystem,
}

impl ResourcesCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ResourcesCollector {
            absent: NotOnThisSystem::new(
                "resources",
                format!(
                    "what this host runs on is read on Linux and on macOS, and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for ResourcesCollector {
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
