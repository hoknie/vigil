use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::{
    AgentBudget, AgentStatus, CollectorRefusal, CollectorState, Counted, FindingsSummary, Silence,
    StoreDropped, StoreStatus,
};

use super::agent::{
    agent, behind, caught_up, collector, collector_degraded, collector_failing, collector_off,
    collector_unavailable, losing, reporter, reporter_failing,
};
use super::store::store;

const SEES_LESS: &str = "the owner of one socket could not be resolved: some rows name no program";

const READING_NOBODY_REFRESHED: &str = "the reading this collector reads was written longer ago than it allows: what it says here may no longer be what the host does";

macro_rules! value {
    ($answer:expr) => {
        serde_json::to_value($answer).expect("an answer is plain data")
    };
}

pub fn watching() -> AgentStatus {
    let mut watching = agent();
    watching.collectors = vec![
        collector("ports", 30, 2, 0),
        collector_degraded("users", 300, SEES_LESS),
        collector_unavailable("processes"),
        collector_failing("persistence"),
        collector_degraded("firewall", 60, READING_NOBODY_REFRESHED),
        collector("resources", 60, 5, 0),
        collector("containers", 60, 3, 0),
        collector("files", 300, 8, 0),
        collector_off(),
    ];
    watching.reporters = vec![reporter("ndjson"), reporter_failing("webhook")];
    watching.buffers = Some(vec![caught_up("ndjson"), behind("webhook")]);
    watching.store = Some(store());
    watching
}

pub fn starting() -> AgentStatus {
    AgentStatus {
        configuration_path: Some("/etc/vigil/vigil.yaml".to_string()),
        version: "0.1.0".into(),
        started_at: "2026-09-09T08:00:00.000Z".into(),
        interval_seconds: 30,
        budget: AgentBudget::default(),
        collectors: Vec::new(),
        reporters: Vec::new(),
        findings: FindingsSummary {
            retained: 0,
            capacity: 500,
            total: 0,
            dropped: 0,
            by_severity: Default::default(),
        },
        silence: Silence::default(),
        store: None,
        buffers: None,
        limitations: Vec::new(),
    }
}

pub fn statuses() -> BTreeMap<String, Value> {
    let watching = watching();
    let mut items = BTreeMap::new();

    items.insert("agent|watching".to_string(), value!(&watching));
    items.insert("agent|starting".to_string(), value!(&starting()));
    for status in &watching.collectors {
        items.insert(
            format!("collector-{}|{}", status.state, status.name),
            value!(status),
        );
    }
    items.insert(
        "collector-degraded|persistence-before-a-reading".to_string(),
        value!(&collector_degraded("persistence", 300, SEES_LESS)),
    );
    for status in &watching.reporters {
        items.insert(format!("reporter|{}", status.name), value!(status));
    }

    items
}

pub fn refusals() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "refusal|users".to_string(),
            value!(&CollectorRefusal::new(
                CollectorState::Degraded,
                "the owner of one socket could not be resolved: some rows name no program",
            )),
        ),
        (
            "refusal|processes".to_string(),
            value!(&CollectorRefusal::new(
                CollectorState::Unavailable,
                "auditd is not running on this host, so nothing is delivering launches",
            )),
        ),
    ])
}

pub fn buffers() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("buffer|caught-up".to_string(), value!(&caught_up("ndjson"))),
        ("buffer|behind".to_string(), value!(&behind("webhook"))),
        ("buffer|losing".to_string(), value!(&losing("webhook"))),
    ])
}

pub fn stores() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("store|written".to_string(), value!(&store())),
        (
            "store|untouched".to_string(),
            value!(&StoreStatus {
                records: Counted::default(),
                bytes: Counted::default(),
                oldest_at: None,
                dropped: StoreDropped::default(),
                damaged: 0,
                journal_path: None,
            }),
        ),
    ])
}
