use std::panic::{AssertUnwindSafe, catch_unwind};

use vigil_collect::{Collector, Health};
use vigil_model::{Finding, Snapshot};
use vigil_rules::{RuleContext, RuleSet, diff};

use crate::helpers::{panic, rfc3339, uuid7};

const SWITCHED_OFF: &str =
    "switched off until its block in the configuration changes or the daemon restarts";

pub struct Watch {
    collector: Box<dyn Collector>,
    rules: RuleSet,
    previous: Option<Snapshot>,
    off: Option<String>,
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
            off: None,
        }
    }

    pub fn name(&self) -> &'static str {
        self.collector.name()
    }

    pub fn switched_off(&self) -> Option<&str> {
        self.off.as_deref()
    }

    pub fn restore(&mut self, baseline: Snapshot) {
        let collector = &self.collector;
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| collector.restore(&baseline))) {
            self.switch_off(payload.as_ref());
        }
        self.previous = Some(baseline);
    }

    pub fn follow(&mut self, collector: Box<dyn Collector>, rules: RuleSet) {
        self.collector = collector;
        self.rules = rules;
        self.off = None;
        if let Some(previous) = self.previous.clone() {
            self.restore(previous);
        }
    }

    pub fn health(&self) -> Health {
        if let Some(why) = &self.off {
            return Health::Unavailable(why.clone());
        }
        let collector = &self.collector;
        catch_unwind(AssertUnwindSafe(|| collector.available())).unwrap_or_else(|payload| {
            Health::Unavailable(format!(
                "{}: the collector panicked: {}",
                collector.name(),
                panic::message(payload.as_ref())
            ))
        })
    }

    pub fn examine(&mut self) -> Result<Health, String> {
        if let Some(why) = &self.off {
            return Err(why.clone());
        }
        let collector = &self.collector;
        catch_unwind(AssertUnwindSafe(|| collector.available()))
            .map_err(|payload| self.switch_off(payload.as_ref()))
    }

    pub fn tick(&mut self) -> Result<Tick, String> {
        if let Some(why) = &self.off {
            return Err(why.clone());
        }
        match catch_unwind(AssertUnwindSafe(|| self.reading())) {
            Ok(outcome) => outcome,
            Err(payload) => Err(self.switch_off(payload.as_ref())),
        }
    }

    fn reading(&mut self) -> Result<Tick, String> {
        let snapshot = self
            .collector
            .collect()
            .map_err(|error| format!("{}: {error}", self.collector.name()))?;

        let Some(previous) = &self.previous else {
            self.previous = Some(snapshot.clone());
            return Ok(Tick {
                findings: Vec::new(),
                changes: 0,
                baseline: true,
                snapshot,
            });
        };

        let changes = diff(previous, &snapshot);
        let findings = self.judge(&changes);
        self.previous = Some(snapshot.clone());

        Ok(Tick {
            findings,
            changes: changes.len(),
            baseline: false,
            snapshot,
        })
    }

    fn switch_off(&mut self, payload: &(dyn std::any::Any + Send)) -> String {
        let why = format!(
            "{}: the collector panicked: {} — {SWITCHED_OFF}",
            self.collector.name(),
            panic::message(payload)
        );
        self.off = Some(why.clone());
        why
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
    use vigil_module::{Module, Settings};

    use super::*;

    struct Scripted(Mutex<Vec<Snapshot>>);

    impl Collector for Scripted {
        fn name(&self) -> &'static str {
            "network"
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
        let mut snapshot = Snapshot::new("network", "2026-09-08T12:00:00.000Z");
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
            vigil_network::Network
                .rules(&Settings::plain(|| "2026-09-09T12:00:00.000Z".to_string())),
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
            vigil_network::Network
                .rules(&Settings::plain(|| "2026-09-09T12:00:00.000Z".to_string())),
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
            vigil_network::Network
                .rules(&Settings::plain(|| "2026-09-09T12:00:00.000Z".to_string())),
        );

        watch.tick().expect("baseline");
        let second = watch.tick().expect("second reading");

        assert_eq!(second.changes, 0);
        assert!(second.findings.is_empty());
    }

    struct Panicking(&'static str);

    impl Collector for Panicking {
        fn name(&self) -> &'static str {
            "network"
        }
        fn available(&self) -> Health {
            panic!("{}", self.0)
        }
        fn collect(&self) -> Result<Snapshot, CollectError> {
            panic!("{}", self.0)
        }
    }

    struct Breaks;

    impl vigil_rules::Rule for Breaks {
        fn name(&self) -> &'static str {
            "breaks"
        }
        fn apply(
            &self,
            _change: &vigil_model::Change,
            _ctx: &mut RuleContext<'_>,
        ) -> Option<Finding> {
            panic!("the rule read a field that is not there")
        }
    }

    fn scripted(readings: Vec<Snapshot>) -> Box<dyn Collector> {
        Box::new(Scripted(Mutex::new(readings)))
    }

    fn network_rules() -> RuleSet {
        vigil_network::Network.rules(&Settings::plain(|| "2026-09-09T12:00:00.000Z".to_string()))
    }

    #[test]
    fn a_collector_that_panics_is_a_failed_reading_that_names_the_panic_and_switches_it_off() {
        let mut watch = Watch::new(
            Box::new(Panicking("index out of range")),
            RuleSet::of(Vec::new()),
        );

        let error = watch.tick().expect_err("a panic is not a reading");

        assert!(
            error.starts_with("network: the collector panicked: index out of range"),
            "{error}"
        );
        assert!(
            error.contains(
                "switched off until its block in the configuration changes or the daemon restarts"
            ),
            "{error}"
        );
        assert_eq!(watch.switched_off(), Some(error.as_str()));
        assert_eq!(
            watch.tick().expect_err("still off"),
            error,
            "a collector that panicked is not read again on its own"
        );
    }

    #[test]
    fn a_panic_that_carries_no_text_still_switches_the_collector_off_with_words() {
        struct Silent;
        impl Collector for Silent {
            fn name(&self) -> &'static str {
                "network"
            }
            fn available(&self) -> Health {
                Health::Ok
            }
            fn collect(&self) -> Result<Snapshot, CollectError> {
                std::panic::panic_any(42_u8)
            }
        }
        let mut watch = Watch::new(Box::new(Silent), RuleSet::of(Vec::new()));

        let error = watch.tick().expect_err("a panic is not a reading");

        assert!(error.contains("a panic with no message"), "{error}");
    }

    #[test]
    fn a_panic_while_asked_for_health_switches_off_only_through_the_round_and_is_never_ok() {
        let mut watch = Watch::new(Box::new(Panicking("no such file")), RuleSet::of(Vec::new()));

        assert!(
            matches!(watch.health(), Health::Unavailable(why) if why.contains("no such file")),
            "the greeting asks for health before anything reads, and a panic there is not ok"
        );
        assert_eq!(watch.switched_off(), None);

        let error = watch.examine().expect_err("a panic is not a health");
        assert!(
            error.contains("the collector panicked: no such file"),
            "{error}"
        );
        assert!(watch.switched_off().is_some());
    }

    #[test]
    fn a_rule_that_panics_leaves_the_reading_before_it_to_compare_against() {
        let mut watch = Watch::new(
            scripted(vec![
                snapshot(&[("tcp|0.0.0.0:443", nginx())]),
                snapshot(&[
                    ("tcp|0.0.0.0:443", nginx()),
                    ("tcp|0.0.0.0:4444", implant()),
                ]),
            ]),
            RuleSet::of(vec![Box::new(Breaks)]),
        );
        watch.restore(snapshot(&[("tcp|0.0.0.0:443", nginx())]));

        let error = watch.tick().expect_err("the rule panicked");
        assert!(
            error.contains("the rule read a field that is not there"),
            "{error}"
        );

        watch.follow(
            scripted(vec![snapshot(&[
                ("tcp|0.0.0.0:443", nginx()),
                ("tcp|0.0.0.0:4444", implant()),
            ])]),
            network_rules(),
        );
        let tick = watch.tick().expect("read again after the block changed");

        assert_eq!(
            tick.findings.len(),
            1,
            "the port that opened on the reading the rule broke on is still news, because the \
             reading it is compared with is the one from before the panic"
        );
        assert_eq!(tick.findings[0].kind.as_str(), "port.listen.new");
    }

    #[test]
    fn a_collector_switched_off_by_a_panic_comes_back_when_its_block_is_taken_up_again() {
        let mut watch = Watch::new(Box::new(Panicking("bad parse")), RuleSet::of(Vec::new()));
        watch.tick().expect_err("it panicked");

        watch.follow(
            scripted(vec![snapshot(&[("tcp|0.0.0.0:443", nginx())])]),
            network_rules(),
        );

        assert_eq!(watch.switched_off(), None);
        assert!(watch.tick().expect("reads again").baseline);
    }

    #[test]
    fn a_collector_that_panics_while_its_baseline_is_handed_back_is_switched_off() {
        struct BreaksOnRestore;
        impl Collector for BreaksOnRestore {
            fn name(&self) -> &'static str {
                "network"
            }
            fn available(&self) -> Health {
                Health::Ok
            }
            fn collect(&self) -> Result<Snapshot, CollectError> {
                Ok(Snapshot::new("network", "2026-09-08T12:00:00.000Z"))
            }
            fn restore(&self, _previous: &Snapshot) {
                panic!("the baseline is from another version")
            }
        }
        let mut watch = Watch::new(Box::new(BreaksOnRestore), RuleSet::of(Vec::new()));

        watch.restore(snapshot(&[("tcp|0.0.0.0:443", nginx())]));

        assert!(
            watch
                .switched_off()
                .is_some_and(|why| why.contains("the baseline is from another version"))
        );
    }

    #[test]
    fn a_collector_that_cannot_read_says_so_instead_of_reporting_an_empty_host() {
        let mut watch = Watch::new(
            Box::new(Scripted(Mutex::new(Vec::new()))),
            RuleSet::of(Vec::new()),
        );

        let error = watch.tick().expect_err("must not be silence");

        assert!(error.contains("network"), "{error}");
    }
}
