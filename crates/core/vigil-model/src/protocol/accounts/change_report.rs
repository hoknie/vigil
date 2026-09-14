use serde::{Deserialize, Serialize};

use super::changed::Changed;
use crate::Rfc3339;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeReport {
    pub acted_at: Rfc3339,
    pub changed: Vec<Changed>,
}

impl ChangeReport {
    pub fn done(&self) -> usize {
        self.changed.iter().filter(|one| one.done).count()
    }

    pub fn refused(&self) -> usize {
        self.changed.len() - self.done()
    }
}
