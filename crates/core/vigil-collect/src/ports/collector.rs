use vigil_model::Snapshot;

use crate::{CollectError, Health, Standing};

pub trait Collector: Send + Sync {
    fn name(&self) -> &'static str;

    fn available(&self) -> Health;

    fn standing(&self) -> Standing {
        Standing::of(self.available())
    }

    fn collect(&self) -> Result<Snapshot, CollectError>;

    fn restore(&self, _previous: &Snapshot) {}
}
