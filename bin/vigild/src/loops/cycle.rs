use vigil_collect::{Collector, Health};
use vigil_model::{Finding, Snapshot};
use vigil_rules::{RuleContext, RuleSet, diff};

use crate::helpers::{rfc3339, uuid7};

pub struct Watch {
    collector: Box<dyn Collector>,
    rules: RuleSet,
    previous: Option<Snapshot>,
}

#[derive(Debug)]
pub struct Tick {
    pub findings: Vec<Finding>,
    pub changes: usize,
    pub baseline: bool,
    pub snapshot: Snapshot,
}

impl Watch {
    pub fn new(collector: Box<dyn Collector>, rules: RuleSet) -> Self {
        Watch {
            collector,
            rules,
            previous: None,
        }
    }

    pub fn name(&self) -> &'static str {
        self.collector.name()
    }

    pub fn restore(&mut self, baseline: Snapshot) {
        self.collector.restore(&baseline);
        self.previous = Some(baseline);
    }

    pub fn health(&self) -> Health {
        self.collector.available()
    }

    pub fn tick(&mut self) -> Result<Tick, String> {
        let snapshot = self
            .collector
            .collect()
            .map_err(|error| format!("{}: {error}", self.collector.name()))?;

        let Some(previous) = self.previous.replace(snapshot.clone()) else {
            return Ok(Tick {
                findings: Vec::new(),
                changes: 0,
                baseline: true,
                snapshot,
            });
        };

        let changes = diff(&previous, &snapshot);
        let findings = self.judge(&changes);

        Ok(Tick {
            findings,
            changes: changes.len(),
            baseline: false,
            snapshot,
        })
    }

    fn judge(&self, changes: &[vigil_model::Change]) -> Vec<Finding> {
        let mut mint = uuid7::mint;
        let mut ctx = RuleContext {
            now: rfc3339::now(),
            mint_event_id: &mut mint,
        };

        self.rules.judge(changes, &mut ctx)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Mutex;
    use vigil_collect::CollectError;
    use vigil_rules::listening_port_rules;

    use super::*;

    struct Scripted(Mutex<Vec<Snapshot>>);

    impl Collector for Scripted {
        fn name(&self) -> &'static str {
            "ports"
        }
        fn available(&self) -> Health {
            Health::Ok
        }
        fn collect(&self) -> Result<Snapshot, CollectError> {
            self.0
                .lock()
                .expect("not poisoned")
                .pop()
                .ok_or_else(|| CollectError::Absent("the script ran out".into()))
        }
    }

    fn snapshot(items: &[(&str, serde_json::Value)]) -> Snapshot {
        let mut snapshot = Snapshot::new("ports", "2026-09-08T12:00:00.000Z");
        for (key, value) in items {
            snapshot.items.insert((*key).to_string(), value.clone());
        }
        snapshot
    }

    fn nginx() -> serde_json::Value {
        json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": 443, "uid": 0, "user": "root",
            "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false, "cmdline": "nginx", "cmdline_redacted": false},
            "owner_resolved": true,
        })
    }

    fn implant() -> serde_json::Value {
        json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": 4444, "uid": 33, "user": "www-data",
            "process": {"exe": "/tmp/.x/nc", "exe_deleted": true, "cmdline": "nc -l", "cmdline_redacted": false},
            "owner_resolved": true,
        })
    }

    #[test]
    fn the_first_tick_is_a_baseline_and_the_second_one_reports() {
        let script = vec![
            snapshot(&[
                ("tcp|0.0.0.0:443", nginx()),
                ("tcp|0.0.0.0:4444", implant()),
            ]),
            snapshot(&[("tcp|0.0.0.0:443", nginx())]),
        ];
        let mut watch = Watch::new(
            Box::new(Scripted(Mutex::new(script))),
            listening_port_rules(),
        );

        let baseline = watch.tick().expect("first reading");
        assert!(baseline.baseline);
        assert_eq!(baseline.snapshot.items.len(), 1);
        assert!(
            baseline.findings.is_empty(),
            "a fresh install must not produce a finding per open port"
        );

        let second = watch.tick().expect("second reading");
        assert!(!second.baseline);
        assert_eq!(second.changes, 1);
        assert_eq!(second.findings.len(), 1);
        assert_eq!(second.findings[0].kind.as_str(), "port.listen.new");
        assert_eq!(second.findings[0].severity.as_str(), "critical");
        assert_eq!(
            second.findings[0].finding_key,
            "port.listen|tcp|0.0.0.0:4444"
        );
    }

    #[test]
    fn a_restored_baseline_means_the_next_reading_is_compared_rather_than_learned() {
        let script = vec![snapshot(&[
            ("tcp|0.0.0.0:443", nginx()),
            ("tcp|0.0.0.0:4444", implant()),
        ])];
        let mut watch = Watch::new(
            Box::new(Scripted(Mutex::new(script))),
            listening_port_rules(),
        );
        watch.restore(snapshot(&[("tcp|0.0.0.0:443", nginx())]));

        let tick = watch.tick().expect("first reading after a restart");

        assert!(!tick.baseline, "a restored daemon is not a fresh install");
        assert_eq!(tick.findings.len(), 1);
        assert_eq!(tick.findings[0].kind.as_str(), "port.listen.new");
    }

    #[test]
    fn a_host_that_did_not_change_produces_nothing_at_all() {
        let script = vec![
            snapshot(&[("tcp|0.0.0.0:443", nginx())]),
            snapshot(&[("tcp|0.0.0.0:443", nginx())]),
        ];
        let mut watch = Watch::new(
            Box::new(Scripted(Mutex::new(script))),
            listening_port_rules(),
        );

        watch.tick().expect("baseline");
        let second = watch.tick().expect("second reading");

        assert_eq!(second.changes, 0);
        assert!(second.findings.is_empty());
    }

    #[test]
    fn a_collector_that_cannot_read_says_so_instead_of_reporting_an_empty_host() {
        let mut watch = Watch::new(
            Box::new(Scripted(Mutex::new(Vec::new()))),
            RuleSet::of(Vec::new()),
        );

        let error = watch.tick().expect_err("must not be silence");

        assert!(error.contains("ports"), "{error}");
    }
}
