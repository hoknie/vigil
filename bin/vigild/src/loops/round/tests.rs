use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::json;
use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{CollectorState, Finding, Snapshot};
use vigil_module::{Module, Settings};
use vigil_store::FileStore;

use crate::budget::Meter;
use crate::socket::{Shared, State};
use crate::types::{Delivery, Due, Policy, Said, Schedule, Startup};

use super::Round;
use crate::loops::Watch;

struct Scripted {
    health: Mutex<Health>,
    readings: Mutex<Vec<Snapshot>>,
    asked: AtomicUsize,
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

fn snapshot(ports: &[u64]) -> Snapshot {
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

fn temporary_directory(name: &str) -> std::path::PathBuf {
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

struct Watching {
    round: Round,
    collector: std::sync::Arc<Scripted>,
    directory: std::path::PathBuf,
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn watching(name: &str, health: Health, readings: Vec<Snapshot>) -> Watching {
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
        },
        collector,
        directory,
    }
}

fn raised(round: &Round) -> Vec<Finding> {
    round.shared.with(|state| state.latest_findings(None))
}

fn shown(round: &Round) -> CollectorState {
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

#[test]
fn a_collector_that_reads_again_is_well_on_that_reading_and_not_five_minutes_later() {
    let mut it = watching(
        "recovered",
        Health::Degraded("the owner of one socket could not be resolved".into()),
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([(
        "ports",
        "degraded:the owner of one socket could not be resolved".to_string(),
    )]);
    it.round.read(0, &mut said);
    *it.collector.health.lock().expect("not poisoned") = Health::Ok;

    it.round.read(0, &mut said);

    assert_eq!(
        shown(&it.round),
        CollectorState::Ok,
        "the reading that found the change knows the collector is well; waiting five minutes to \
         say so leaves a person looking at a red mark and at the findings that disprove it"
    );
    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "and the finding that closes the complaint is raised there and then: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_reading_the_console_asked_for_is_taken_at_once_and_the_period_starts_again_from_it() {
    let mut it = watching(
        "asked",
        Health::Ok,
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);
    it.round.schedule.advance(0, Instant::now());
    it.round
        .shared
        .with(|state| state.ask_for_a_reading("ports"));

    it.round.read_what_the_console_asked_for(&mut said);

    assert!(
        it.round.shared.with(|state| state
            .snapshot("ports")
            .is_some_and(|reading| reading.items.contains_key("tcp|0.0.0.0:4444"))),
        "a person who just changed the host is shown the host as it is now, not as it was \
         when the period last came round"
    );
    assert!(
        it.round.schedule.waiting(0, Instant::now()) > std::time::Duration::from_secs(29),
        "and the next reading is a whole period after this one, not the slot that was due anyway"
    );

    it.round.read_what_the_console_asked_for(&mut said);
    assert!(
        it.collector
            .readings
            .lock()
            .expect("not poisoned")
            .is_empty(),
        "both readings were taken, and a turn with nothing asked for reads nothing more"
    );
}

#[test]
fn a_reading_asked_for_a_collector_this_agent_does_not_watch_reads_nothing() {
    let mut it = watching("unwatched", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round
        .shared
        .with(|state| state.ask_for_a_reading("users"));

    it.round.read_what_the_console_asked_for(&mut said);

    assert_eq!(
        it.collector.readings.lock().expect("not poisoned").len(),
        1,
        "users is switched off on this host, and asking for it is not a reason to read ports"
    );
}

#[test]
fn a_reading_that_found_nothing_does_not_pay_for_a_second_look_at_the_collector() {
    let mut it = watching(
        "quiet",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);
    let after_the_baseline = it.collector.asked.load(Ordering::Relaxed);

    it.round.read(0, &mut said);

    assert_eq!(
        it.collector.asked.load(Ordering::Relaxed),
        after_the_baseline,
        "asking a collector how it is costs as much as a reading on the collectors where it \
         walks /proc twice; on a host where nothing moved, nothing has to be asked"
    );
}

#[test]
fn a_collector_that_could_not_read_and_then_read_is_looked_at_again_without_waiting() {
    let mut it = watching("failed", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);
    let after_the_failure = it.collector.asked.load(Ordering::Relaxed);
    assert_eq!(
        shown(&it.round),
        CollectorState::Degraded,
        "a collector that could not read is not a collector reporting an empty host"
    );
    *it.collector.readings.lock().expect("not poisoned") = vec![snapshot(&[443])];

    it.round.read(0, &mut said);

    assert!(
        it.collector.asked.load(Ordering::Relaxed) > after_the_failure,
        "a collector that failed and read again has changed state, whatever its snapshot says"
    );
    assert_eq!(shown(&it.round), CollectorState::Ok);
}

#[test]
fn a_reading_of_a_collector_that_is_still_well_says_nothing_about_its_health() {
    let mut it = watching(
        "unchanged",
        Health::Ok,
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);

    it.round.read(0, &mut said);

    assert!(
        raised(&it.round)
            .iter()
            .all(|finding| !finding.kind.as_str().starts_with("agent.collector")),
        "a change on the host is not news about the agent: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.title.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        raised(&it.round).len(),
        1,
        "and the finding about the port that appeared is the only one"
    );
}

fn open_in_the_journal(round: &Round) -> Option<Finding> {
    use vigil_store::Store;

    round
        .store
        .open_finding("agent.collector|ports", "agent.collector.degraded")
        .expect("readable")
}

#[test]
fn a_reading_that_failed_is_a_finding_and_not_only_a_line_in_the_daemons_log() {
    let mut it = watching("failed-once", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);

    it.round.read(0, &mut said);

    let findings = raised(&it.round);
    let about_it = findings
        .iter()
        .find(|finding| finding.kind.as_str() == "agent.collector.degraded")
        .expect("a collector that cannot read has to say so where the findings go");
    assert_eq!(about_it.finding_key, "agent.collector|ports");
    assert!(
        about_it
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "error" && evidence.value.contains("ports")),
        "the finding carries the same error the screen shows: {:?}",
        about_it.evidence
    );
    assert!(
        about_it
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "cost" && evidence.value.contains("failed reading")),
        "and the same counters: {:?}",
        about_it.evidence
    );
}

#[test]
fn a_collector_that_fails_every_reading_raises_one_finding_and_not_one_a_tick() {
    let mut it = watching("failing", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);

    for _ in 0..10 {
        it.round.read(0, &mut said);
    }

    let about_it = raised(&it.round)
        .into_iter()
        .filter(|finding| finding.kind.as_str() == "agent.collector.degraded")
        .count();
    assert_eq!(
        about_it, 1,
        "a failure that repeats every tick is one thing wrong with the host, and a finding a \
         tick is a generator of rubbish"
    );
    assert_eq!(
        open_in_the_journal(&it.round)
            .expect("the finding is in the journal")
            .occurrences,
        1,
        "the same failure repeating is not a new sighting to write down either: what changes \
         the record is a reason that changed"
    );
}

#[test]
fn a_reading_that_went_through_closes_the_finding_about_the_ones_that_did_not() {
    let mut it = watching("failed-then-read", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);
    for _ in 0..3 {
        it.round.read(0, &mut said);
    }
    assert!(open_in_the_journal(&it.round).is_some(), "it stood first");

    *it.collector.readings.lock().expect("not poisoned") = vec![snapshot(&[443])];
    it.round.read(0, &mut said);

    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "a collector that read again closes what was raised when it could not: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        open_in_the_journal(&it.round).is_none(),
        "and the journal a restart reads has nothing open about it any more"
    );
}

#[test]
fn a_reason_that_changed_is_written_down_without_alerting_a_second_time() {
    let mut it = watching("two-reasons", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);

    said.failing("ports", "something else entirely");
    it.round.read(0, &mut said);

    assert_eq!(
        open_in_the_journal(&it.round)
            .expect("still open")
            .occurrences,
        2,
        "a reason that changed is written into the history of the same finding"
    );
    assert_eq!(
        raised(&it.round)
            .iter()
            .filter(|finding| finding.kind.as_str() == "agent.collector.degraded")
            .count(),
        1,
        "and it is not a second thing to tell a person about: one collector, one complaint"
    );
}

#[test]
fn a_complaint_a_previous_run_left_open_is_closed_by_the_first_reading_that_goes_through() {
    let mut it = watching("from-the-last-run", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("ports", "ok".to_string())]);
    said.opened(
        "ports",
        "the reading failed — ports: /proc/net/tcp: permission denied".to_string(),
    );

    it.round.read(0, &mut said);

    let closed = raised(&it.round)
        .into_iter()
        .find(|finding| finding.kind.as_str() == "agent.collector.recovered")
        .expect("the first reading that went through is the proof it is watching again");
    assert!(
        closed
            .evidence
            .iter()
            .any(|evidence| evidence.value.contains("permission denied")),
        "and it quotes the complaint it closes, which came out of the journal: {:?}",
        closed.evidence
    );
}

#[test]
fn a_collector_that_came_back_is_reported_once_and_not_on_every_reading_after_it() {
    let mut it = watching(
        "came-back",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    said.opened("ports", "degraded — one socket had no owner".to_string());

    it.round.read(0, &mut said);
    it.round.read(0, &mut said);
    it.round.read(0, &mut said);

    assert_eq!(
        raised(&it.round)
            .iter()
            .filter(|finding| finding.kind.as_str() == "agent.collector.recovered")
            .count(),
        1,
        "coming back happened once; a line a reading would be the noise the pair exists to avoid"
    );
}

#[test]
fn a_collector_that_goes_unwell_while_the_agent_runs_is_reported_and_then_closed() {
    let mut it = watching(
        "mid-run",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);

    *it.collector.health.lock().expect("not poisoned") =
        Health::Degraded("the owner of one socket could not be resolved".into());
    it.round.take_health(true, &mut said);
    let complained = raised(&it.round);
    *it.collector.health.lock().expect("not poisoned") = Health::Ok;
    it.round.take_health(true, &mut said);

    assert!(
        complained
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.degraded"),
        "a collector that stops seeing part of a host mid-run says so: {:?}",
        complained
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "and the pass that finds it well again closes what that raised"
    );
    assert!(
        open_in_the_journal(&it.round).is_none(),
        "in the journal too, which is what the next start reads"
    );
}
