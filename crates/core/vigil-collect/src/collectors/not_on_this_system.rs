use vigil_model::Snapshot;

use crate::ports::Collector;
use crate::types::{CollectError, Health};

pub struct NotOnThisSystem {
    name: &'static str,
    reason: String,
}

impl NotOnThisSystem {
    pub fn new(name: &'static str, reason: impl Into<String>) -> Self {
        NotOnThisSystem {
            name,
            reason: reason.into(),
        }
    }
}

impl Collector for NotOnThisSystem {
    fn name(&self) -> &'static str {
        self.name
    }

    fn available(&self) -> Health {
        Health::Unavailable(self.reason.clone())
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        Err(CollectError::Absent(self.reason.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_subject_this_system_cannot_be_read_for_says_why_and_never_reads_as_empty() {
        let collector = NotOnThisSystem::new(
            "containers",
            "containers on macOS run inside a virtual machine",
        );

        assert_eq!(collector.name(), "containers");
        assert_eq!(
            collector.available(),
            Health::Unavailable("containers on macOS run inside a virtual machine".into())
        );
        assert!(
            collector.collect().is_err(),
            "an empty reading would be stored as a host where nothing runs"
        );
    }
}
