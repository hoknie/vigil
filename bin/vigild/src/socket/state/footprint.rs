use vigil_model::{AgentBudget, StoreStatus};

#[derive(Debug, Clone, Default)]
pub(super) struct Footprint {
    pub budget: AgentBudget,
    pub store: Option<StoreStatus>,
}
