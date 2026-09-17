use serde::{Deserialize, Serialize};

use super::control_target::ControlTarget;
use super::controlled::Controlled;
use super::controlling::Controlling;
use crate::Rfc3339;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlReport {
    #[serde(default)]
    pub target: ControlTarget,
    pub controlling: Controlling,
    pub acted_at: Rfc3339,
    pub controlled: Vec<Controlled>,
}

impl ControlReport {
    pub fn done(&self) -> usize {
        self.controlled.iter().filter(|one| one.done).count()
    }

    pub fn refused(&self) -> usize {
        self.controlled.len() - self.done()
    }
}
