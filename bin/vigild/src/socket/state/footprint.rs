use vigil_model::{AgentBudget, BufferStatus, StoreStatus};

#[derive(Debug, Clone, Default)]
pub(super) struct Footprint {
    pub budget: AgentBudget,
    pub store: Option<StoreStatus>,
    pub buffers: Option<Vec<BufferStatus>>,
}
