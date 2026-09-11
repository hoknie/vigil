use vigil_collect::Health;
use vigil_model::Snapshot;

use crate::socket::State;
use crate::types::{Reading, Startup};

use super::host;

pub fn state() -> State {
    State::new(
        Startup {
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), 30u32)].into_iter().collect(),
        },
        &[("ports", Health::Ok)],
        &["ndjson".to_string()],
        &[],
    )
}

pub fn reading(snapshot: Snapshot) -> Reading {
    reading_of("ports", snapshot)
}

pub fn reading_of(collector: &'static str, snapshot: Snapshot) -> Reading {
    Reading {
        collector,
        at: "2026-09-09T09:00:00.000Z".into(),
        duration_ms: 5,
        every_seconds: 30,
        next_run_at: "2026-09-09T09:00:30.000Z".into(),
        skipped: 0,
        snapshot,
    }
}
