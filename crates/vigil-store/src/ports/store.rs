use vigil_model::{Finding, Snapshot};

use crate::{Kept, Recorded, StoreError};

pub trait Store: Send + Sync {
    fn baseline(&self, source: &str) -> Result<Option<Snapshot>, StoreError>;

    fn set_baseline(&self, snapshot: &Snapshot) -> Result<(), StoreError>;

    fn forget_baseline(&self, source: &str) -> Result<bool, StoreError>;

    fn record(&self, finding: &Finding) -> Result<Recorded, StoreError>;

    fn open_findings(&self, limit: usize) -> Result<Vec<Finding>, StoreError>;

    fn resolve(&self, finding_key: &str, kind: &str, at: &str) -> Result<bool, StoreError>;

    fn prune(&self, older_than: &str) -> Result<u64, StoreError>;

    fn kept(&self) -> Result<Kept, StoreError>;
}
