use std::collections::BTreeMap;

use serde_json::Value;
use vigil_collect::Health;
use vigil_model::{BufferStatus, Counted, Severity, StoreDropped, StoreStatus};

use crate::socket::State;
use crate::types::Startup;

use super::agent::period;
use super::{finding, finding_of, host, reading, reading_of, snapshot};

const DEGRADED: &str = "the owner of one socket could not be resolved: some rows name no program";

const UNAVAILABLE: &str = "auditd is not running on this host, so nothing is delivering launches";

const FAILED: &str = "not permitted to read /proc/modules";

const NO_RULESET_YET: &str = "the ruleset in /var/lib/vigil/firewall/ruleset.json was written 214 seconds ago, more than the 120 this collector allows: what it says about this host may have been true and no longer is";

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

pub fn behind() -> BufferStatus {
    BufferStatus {
        receiver: "webhook".into(),
        pending: 12,
        pending_ceiling: 500,
        bytes: 8_792,
        bytes_ceiling: 4 * 1024 * 1024,
        dropped_total: 0,
        oldest_at: Some("2026-09-09T08:41:02.000Z".into()),
    }
}

pub fn caught_up() -> BufferStatus {
    BufferStatus {
        receiver: "ndjson".into(),
        pending_ceiling: 500,
        bytes_ceiling: 4 * 1024 * 1024,
        ..BufferStatus::default()
    }
}

pub fn losing() -> BufferStatus {
    BufferStatus {
        receiver: "ndjson-2".into(),
        pending: 500,
        pending_ceiling: 500,
        bytes: 369_664,
        bytes_ceiling: 4 * 1024 * 1024,
        dropped_total: 41,
        oldest_at: Some("2026-09-08T22:14:09.000Z".into()),
    }
}

pub fn every_state() -> State {
    let mut state = State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [
                "ports",
                "users",
                "persistence",
                "processes",
                "firewall",
                "resources",
            ]
            .into_iter()
            .map(|name| (name.to_string(), period(name)))
            .collect(),
            killing_from_the_console: false,
        },
        &[
            ("ports", Health::Ok),
            ("users", Health::Degraded(DEGRADED.to_string())),
            ("processes", Health::Unavailable(UNAVAILABLE.to_string())),
            ("persistence", Health::Ok),
            ("firewall", Health::Degraded(NO_RULESET_YET.to_string())),
            ("resources", Health::Ok),
            ("containers", Health::Ok),
            ("files", Health::Ok),
        ],
        &["ndjson".to_string(), "webhook".to_string()],
        &["launches".to_string()],
    );

    state.record_reading(reading(snapshot()));
    state.record_reading(reading_of("persistence", snapshot()));
    state.record_reading(reading_of("resources", snapshot()));
    state.record_reading(reading_of("containers", snapshot()));
    state.record_reading(reading_of("files", snapshot()));
    state.record_failure("persistence", "2026-09-09T09:00:30.000Z".into(), FAILED);
    state.record_budget(Some(0.02), Some(12_288));
    state.record_store(store());
    state.record_buffers(vec![caught_up(), behind()]);
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
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: BTreeMap::new(),
            killing_from_the_console: false,
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

pub fn buffers() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("buffer|behind".to_string(), value(&behind())),
        ("buffer|caught-up".to_string(), value(&caught_up())),
        ("buffer|losing".to_string(), value(&losing())),
    ])
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
