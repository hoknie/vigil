use vigil_collect::Health;
use vigil_model::Snapshot;

use crate::modules::every_seconds_of;
use crate::socket::State;
use crate::types::{Reading, Startup};

use super::host;

pub fn period(collector: &str) -> u32 {
    every_seconds_of(collector)
        .unwrap_or_else(|| panic!("{collector} is not a collector this build ships"))
}

pub fn state() -> State {
    State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), period("ports"))]
                .into_iter()
                .collect(),
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
        every_seconds: period(collector),
        next_run_at: "2026-09-09T09:00:30.000Z".into(),
        skipped: 0,
        snapshot,
    }
}
