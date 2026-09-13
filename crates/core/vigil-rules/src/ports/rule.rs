use vigil_model::{Change, Finding, Rfc3339, Uuid7};

pub struct RuleContext<'a> {
    pub now: Rfc3339,
    pub mint_event_id: &'a mut dyn FnMut() -> Uuid7,
}

pub trait Rule {
    fn name(&self) -> &'static str;

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding>;
}
