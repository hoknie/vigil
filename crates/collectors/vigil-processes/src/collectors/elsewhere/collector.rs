use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct ProcessesCollector {
    absent: NotOnThisSystem,
}

impl ProcessesCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ProcessesCollector {
            absent: NotOnThisSystem::new(
                "processes",
                format!(
                    "the programs running here are read on Linux and on macOS, and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for ProcessesCollector {
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
