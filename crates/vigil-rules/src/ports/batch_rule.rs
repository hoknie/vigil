use std::collections::BTreeSet;

use vigil_model::{Change, Finding};

use super::rule::RuleContext;
pub struct Batch {
    pub findings: Vec<Finding>,
    pub claimed: BTreeSet<String>,
}

impl Batch {
    pub fn silent() -> Self {
        Batch {
            findings: Vec::new(),
            claimed: BTreeSet::new(),
        }
    }
}
pub trait BatchRule {
    fn name(&self) -> &'static str;

    fn apply(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Batch;
}
