use super::execution::Execution;

pub struct AuditReading {
    pub executions: Vec<Execution>,
    pub unnamed: usize,
    pub consumed: usize,
}
