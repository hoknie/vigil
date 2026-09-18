use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::json;
use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{CollectorState, Finding, Snapshot};
use vigil_module::{Module, Settings};
use vigil_store::FileStore;

use crate::budget::Meter;
use crate::config::{Followed, Silences};
use crate::socket::{Shared, State};
use crate::types::{Delivery, Due, Policy, Schedule, Startup};

use crate::loops::{Round, Watch};

pub struct Scripted {
    pub health: Mutex<Health>,
    pub readings: Mutex<Vec<Snapshot>>,
    pub asked: AtomicUsize,
}

impl Scripted {
    fn new(health: Health, readings: Vec<Snapshot>) -> Self {
        Scripted {
            health: Mutex::new(health),
            readings: Mutex::new(readings),
            asked: AtomicUsize::new(0),
        }
    }
}

impl Collector for Scripted {
    fn name(&self) -> &'static str {
        "ports"
    }

    fn available(&self) -> Health {
        self.asked.fetch_add(1, Ordering::Relaxed);
        self.health.lock().expect("not poisoned").clone()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        self.readings
            .lock()
            .expect("not poisoned")
            .pop()
            .ok_or_else(|| CollectError::Absent("the script ran out".into()))
    }
}

pub fn snapshot(ports: &[u64]) -> Snapshot {
    let mut snapshot = Snapshot::new("ports", "2026-09-11T09:00:00.000Z");
    for port in ports {
        snapshot.items.insert(
            format!("tcp|0.0.0.0:{port}"),
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": port, "uid": 0, "user": "root",
                "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false, "cmdline": "nginx", "cmdline_redacted": false},
                "owner_resolved": true,
            }),
        );
    }
    snapshot
}

pub fn temporary_directory(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "vigild-round-{}-{name}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&directory);
    directory
}

pub struct Watching {
    pub round: Round,
    pub collector: std::sync::Arc<Scripted>,
    pub directory: std::path::PathBuf,
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

pub fn watching(name: &str, health: Health, readings: Vec<Snapshot>) -> Watching {
    struct Handed(std::sync::Arc<Scripted>);

    impl Collector for Handed {
        fn name(&self) -> &'static str {
            self.0.name()
        }
        fn available(&self) -> Health {
            self.0.available()
        }
        fn collect(&self) -> Result<Snapshot, CollectError> {
            self.0.collect()
        }
    }

    let collector = std::sync::Arc::new(Scripted::new(health.clone(), readings));
    let directory = temporary_directory(name);
    let shared = Shared::new(State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: crate::socket::fixture::host(),
            started_at: "2026-09-11T09:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), 30)].into_iter().collect(),
            killing_from_the_console: false,
            accounts_from_the_console: false,
            units_from_the_console: false,
        },
        &[("ports", health)],
        &[],
        &[],
    ));

    Watching {
        round: Round {
            watches: vec![Watch::new(
                Box::new(Handed(collector.clone())),
                vigil_network::Ports
                    .rules(&Settings::plain(|| "2026-09-09T12:00:00.000Z".to_string())),
            )],
            store: FileStore::open(&directory).expect("opens"),
            policy: Policy::new(Vec::new()),
            delivery: Delivery::new(
                crate::socket::fixture::host(),
                Vec::new(),
                Vec::new(),
                shared.clone(),
            ),
            shared,
            schedule: Schedule::of(vec![Due::new("ports", 30, Instant::now())]),
            meter: Meter::default(),
            opening: Vec::new(),
            standing: Vec::new(),
            followed: Followed::nothing(),
            silences: Silences::nothing(),
        },
        collector,
        directory,
    }
}

pub fn raised(round: &Round) -> Vec<Finding> {
    round.shared.with(|state| state.latest_findings(None))
}

pub fn shown(round: &Round) -> CollectorState {
    round.shared.with(|state| {
        state
            .agent()
            .collectors
            .into_iter()
            .find(|collector| collector.name == "ports")
            .expect("the collector the round watches")
            .state
    })
}

pub fn open_in_the_journal(round: &Round) -> Option<Finding> {
    use vigil_store::Store;

    round
        .store
        .open_finding("agent.collector|ports", "agent.collector.degraded")
        .expect("readable")
}
