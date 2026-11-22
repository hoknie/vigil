use vigil_collect::{CollectError, Collector, Health, Standing};
use vigil_model::{Rfc3339, Snapshot};

use super::health::standing;
use super::reading::reading;

pub struct UsersCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl UsersCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        UsersCollector { now: Box::new(now) }
    }
}

impl Collector for UsersCollector {
    fn name(&self) -> &'static str {
        "users"
    }

    fn available(&self) -> Health {
        standing().health
    }

    fn standing(&self) -> Standing {
        standing()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        reading(&(self.now)())
    }
}
