use vigil_model::{Rfc3339, Snapshot};

pub struct Reading {
    pub collector: &'static str,
    pub at: Rfc3339,
    pub duration_ms: u64,
    pub every_seconds: u32,
    pub next_run_at: Rfc3339,
    pub skipped: u64,
    pub snapshot: Snapshot,
}
