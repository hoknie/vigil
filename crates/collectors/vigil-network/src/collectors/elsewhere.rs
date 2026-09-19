use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct NetworkCollector {
    absent: NotOnThisSystem,
}

impl NetworkCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        NetworkCollector {
            absent: NotOnThisSystem::new(
                "network",
                format!(
                    "the sockets of this host are read on Linux and on macOS, and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for NetworkCollector {
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
