use vigil_model::Change;
use vigil_rules::{Batch, BatchRule, RuleContext};

use crate::helpers::silenced;

pub struct EngineDidNotAnswer;

impl BatchRule for EngineDidNotAnswer {
    fn name(&self) -> &'static str {
        "engine_did_not_answer"
    }

    fn apply(&self, changes: &[Change], _ctx: &mut RuleContext<'_>) -> Batch {
        Batch {
            findings: Vec::new(),
            claimed: silenced(changes),
        }
    }
}
