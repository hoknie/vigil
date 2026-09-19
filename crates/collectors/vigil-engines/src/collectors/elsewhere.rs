use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

use crate::types::Watching;

pub struct EnginesCollector {
    absent: NotOnThisSystem,
}

impl EnginesCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static, watching: Watching) -> Self {
        EnginesCollector {
            absent: NotOnThisSystem::new(
                crate::parsers::SOURCE,
                format!(
                    "what the {} of a host hold is read from the files a job of this package \
                     writes on Linux and on macOS, and this is {}",
                    watching.engines.join(" and "),
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for EnginesCollector {
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
