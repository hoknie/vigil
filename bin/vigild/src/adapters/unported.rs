use vigil_collect::{CollectError, Collector, Health};
use vigil_model::Snapshot;

pub struct Unported {
    name: &'static str,
    why: String,
}

impl Unported {
    pub fn new(name: &'static str, why: impl Into<String>) -> Self {
        Unported {
            name,
            why: why.into(),
        }
    }
}

impl Collector for Unported {
    fn name(&self) -> &'static str {
        self.name
    }

    fn available(&self) -> Health {
        Health::Unavailable(format!(
            "this build reads nothing of it on {}: {}",
            std::env::consts::OS,
            self.why
        ))
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        Err(CollectError::Absent(format!(
            "a {} collector for {}",
            self.name,
            std::env::consts::OS
        )))
    }
}
