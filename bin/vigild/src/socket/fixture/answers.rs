use std::collections::BTreeMap;

use serde_json::Value;
use vigil_collect::Health;
use vigil_model::{Counted, Severity, StoreDropped, StoreStatus};

use crate::socket::State;
use crate::types::Startup;

use super::agent::period;
use super::{finding, finding_of, host, reading, reading_of, snapshot};

const DEGRADED: &str = "the owner of one socket could not be resolved: some rows name no program";

const UNAVAILABLE: &str = "auditd is not running on this host, so nothing is delivering launches";

const FAILED: &str = "not permitted to read /proc/modules";

pub fn store() -> StoreStatus {
    StoreStatus {
        records: Counted {
            held: 1_284,
            ceiling: 10_000,
        },
        bytes: Counted {
            held: 2_202_009,
            ceiling: 16 * 1024 * 1024,
        },
        oldest_at: Some("2026-08-27T04:11:53.000Z".into()),
        dropped: StoreDropped {
            at_the_ceiling: 0,
            past_the_window: 41,
        },
        damaged: 3,
        journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
    }
}

pub fn every_state() -> State {
    let mut state = State::new(
        Startup {
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: ["ports", "users", "persistence", "processes"]
                .into_iter()
                .map(|name| (name.to_string(), period(name)))
                .collect(),
        },
        &[
            ("ports", Health::Ok),
            ("users", Health::Degraded(DEGRADED.to_string())),
            ("processes", Health::Unavailable(UNAVAILABLE.to_string())),
            ("persistence", Health::Ok),
        ],
        &["ndjson".to_string(), "webhook".to_string()],
        &["launches".to_string()],
    );

    state.record_reading(reading(snapshot()));
    state.record_reading(reading_of("persistence", snapshot()));
    state.record_failure("persistence", "2026-09-09T09:00:30.000Z".into(), FAILED);
    state.record_budget(Some(0.02), Some(12_288));
    state.record_store(store());
    state.record_findings(&[
        finding("a new listening port"),
        finding_of(Severity::Low, "a listening port closed"),
        finding_of(Severity::Low, "a user logged in from a new address"),
    ]);
    state.record_policy(
        vec!["port.listen|tcp|10.0.0.5:* — the staging api, expected".to_string()],
        1,
    );
    state.record_delivery(
        "webhook",
        "2026-09-09T09:00:00.000Z".into(),
        Some("the receiver answered 503".to_string()),
    );
    state.record_delivery("ndjson", "2026-09-09T09:00:00.000Z".into(), None);

    state
}

pub fn before_the_first_reading() -> State {
    State::new(
        Startup {
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: BTreeMap::new(),
        },
        &[],
        &[],
        &[],
    )
}

pub fn statuses() -> BTreeMap<String, Value> {
    let agent = every_state().agent();
    let mut items = BTreeMap::new();

    items.insert("agent|watching".to_string(), value(&agent));
    items.insert(
        "agent|starting".to_string(),
        value(&before_the_first_reading().agent()),
    );
    for collector in &agent.collectors {
        items.insert(
            format!("collector-{}|{}", collector.state, collector.name),
            value(collector),
        );
    }
    for reporter in &agent.reporters {
        items.insert(format!("reporter|{}", reporter.name), value(reporter));
    }

    items
}

pub fn refusals() -> BTreeMap<String, Value> {
    let state = every_state();
    let mut items = BTreeMap::new();

    for name in ["users", "processes", "persistence"] {
        if let Some(refusal) = state.refusal(name) {
            items.insert(format!("refusal|{name}"), value(&refusal));
        }
    }

    items
}

pub fn stores() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("store|written".to_string(), value(&store())),
        (
            "store|untouched".to_string(),
            value(&StoreStatus::default()),
        ),
    ])
}

fn value<T: serde::Serialize>(answer: &T) -> Value {
    serde_json::to_value(answer).expect("an answer is plain data")
}
