use vigil_model::{Rfc3339, Snapshot};

use super::{health, reading};
use vigil_collect::{CollectError, Collector, Health};

pub struct UsersCollector {
    pub(super) now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl UsersCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        UsersCollector { now: Box::new(now) }
    }

    fn taken_at(&self) -> Rfc3339 {
        (self.now)()
    }
}

impl Collector for UsersCollector {
    fn name(&self) -> &'static str {
        "users"
    }

    fn available(&self) -> Health {
        health::health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        reading::reading(&self.taken_at())
    }
}
