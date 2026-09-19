use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct LaunchesCollector {
    absent: NotOnThisSystem,
}

impl LaunchesCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static, _keep_arguments: bool) -> Self {
        LaunchesCollector {
            absent: NotOnThisSystem::new(
                "launches",
                format!(
                    "what people run is read from auditd on Linux and from eslogger on macOS, \
                     and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for LaunchesCollector {
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
