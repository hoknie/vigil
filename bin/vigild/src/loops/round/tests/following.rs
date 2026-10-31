use std::time::{Duration, Instant};

use serde_json::{Value, json};
use vigil_collect::{CollectError, Collector, Health};
use vigil_model::Snapshot;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;

use super::harness::{about_the_host, raised, snapshot, temporary_directory, watching};
use crate::config::{Followed, Stamp, load};
use crate::types::Said;

const WATCHING_HOSTS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n";

const WATCHING_SUDOERS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n    - \"/etc/sudoers\"\n";

struct Listed(Value);

impl Collector for Listed {
    fn name(&self) -> &'static str {
        "network"
    }
    fn available(&self) -> Health {
        Health::Ok
    }
    fn collect(&self) -> Result<Snapshot, CollectError> {
        let mut reading = Snapshot::new("network", "2026-09-17T12:00:00.000Z");
        for path in self.0["paths"].as_array().into_iter().flatten() {
            reading.items.insert(
                format!("file|{}", path.as_str().unwrap_or_default()),
                json!({}),
            );
        }
        Ok(reading)
    }
}

struct FollowingTheFile;

impl Module for FollowingTheFile {
    fn name(&self) -> &'static str {
        "network"
    }
    fn subject(&self) -> &'static str {
        "a module that takes its list from the file while the daemon runs"
    }
    fn every_seconds(&self) -> u32 {
        30
    }
    fn settings_key(&self) -> Option<&'static str> {
        Some("files")
    }
    fn follows_the_file(&self) -> bool {
        true
    }
    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        Ok(Box::new(Listed(settings.said().clone())))
    }
    fn rules(&self, _settings: &Settings) -> RuleSet {
        RuleSet::of(Vec::new())
    }
    fn families(&self) -> &[&'static str] {
        &["file"]
    }
}

fn written(path: &std::path::Path, text: &str) {
    let writing = path.with_extension("yaml.writing");
    std::fs::write(&writing, text).expect("writes");
    std::fs::rename(&writing, path).expect("moves into place, as the console does");
}

fn read_now(round: &crate::loops::Round) -> Vec<String> {
    round.shared.with(|state| {
        state
            .snapshot("network")
            .map(|reading| reading.items.keys().cloned().collect())
            .unwrap_or_default()
    })
}

#[test]
fn a_path_written_into_the_file_is_read_on_the_round_after_it_and_not_after_a_restart() {
    let mut it = watching("following-good", Health::Ok, vec![snapshot(&[443])]);
    let directory = temporary_directory("following-good-configuration");
    std::fs::create_dir_all(&directory).expect("a directory");
    let path = directory.join("vigil.yaml");
    written(&path, WATCHING_HOSTS);
    let name = path.to_str().expect("utf-8");
    it.round.followed = Followed::of(
        name,
        Stamp::of(name),
        &load(name).expect("loads"),
        vec![Box::new(FollowingTheFile)],
        &["network"],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    it.round.schedule.advance(0, Instant::now());

    written(&path, WATCHING_SUDOERS);
    it.round.follow_the_file(&mut said);

    assert_eq!(
        read_now(&it.round),
        vec![
            "file|/etc/hosts".to_string(),
            "file|/etc/sudoers".to_string()
        ],
        "the console said the path is watched, and the reading a person opens next shows it"
    );
    assert!(
        it.round.schedule.waiting(0, Instant::now()) > Duration::from_secs(29),
        "the reading taken for the change starts the period again"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_bad_edit_leaves_the_collector_that_was_running_in_place_and_raises_nothing() {
    let mut it = watching(
        "following-bad",
        Health::Ok,
        vec![snapshot(&[443, 8080]), snapshot(&[443])],
    );
    let directory = temporary_directory("following-bad-configuration");
    std::fs::create_dir_all(&directory).expect("a directory");
    let path = directory.join("vigil.yaml");
    written(&path, WATCHING_HOSTS);
    let name = path.to_str().expect("utf-8");
    it.round.followed = Followed::of(
        name,
        Stamp::of(name),
        &load(name).expect("loads"),
        vec![Box::new(FollowingTheFile)],
        &["network"],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    let before = raised(&it.round).len();

    written(&path, "files:\n  paths:\n    - \"etc/sudoers\"\n");
    for _ in 0..3 {
        it.round.follow_the_file(&mut said);
    }
    assert_eq!(
        raised(&it.round).len(),
        before,
        "a typo in the file is a line in the log, not a finding on every round it stays there"
    );
    it.round.read(0, &mut said);

    assert!(
        read_now(&it.round).contains(&"tcp|0.0.0.0:8080".to_string()),
        "the reading after a bad edit is taken by the collector that ran before it: {:?}",
        read_now(&it.round)
    );

    written(&path, WATCHING_SUDOERS);
    it.round.follow_the_file(&mut said);
    assert_eq!(
        read_now(&it.round),
        vec![
            "file|/etc/hosts".to_string(),
            "file|/etc/sudoers".to_string()
        ],
        "the file that loads again is taken up on the round after it is written"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn an_object_silenced_in_the_directory_is_not_reported_from_the_next_round_on() {
    let mut it = watching(
        "following-silence",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443, 8080])],
    );
    let directory = temporary_directory("following-silence-configuration");
    std::fs::create_dir_all(directory.join("suppressions")).expect("a directory");
    let path = directory.join("vigil.yaml");
    written(&path, "suppressions_path: suppressions\n");
    let name = path.to_str().expect("utf-8");
    it.round.silences = crate::config::Silences::of(name, &load(name).expect("loads"));
    let mut said = Said::about([("network", "ok".to_string())]);

    written(
        &directory.join("suppressions").join("console.yaml"),
        "suppressions:\n  - finding_key_prefix: \"port.listen|\"\n    reason: every port is expected here\n",
    );
    it.round.follow_the_file(&mut said);

    let silence = it.round.shared.with(|state| state.agent().silence);
    assert_eq!(
        silence.suppressions,
        vec!["port.listen|* — every port is expected here".to_string()],
        "the console reads what is in force from the status, and it is in force now"
    );
    it.round.read(0, &mut said);
    it.round.read(0, &mut said);
    assert!(
        about_the_host(&it.round).is_empty(),
        "the port that opened after the entry was written is covered by it: {:?}",
        about_the_host(&it.round)
    );
    assert_eq!(it.round.policy.suppressed(), 1);
}

struct PanicsUntilFollowed;

impl Collector for PanicsUntilFollowed {
    fn name(&self) -> &'static str {
        "network"
    }
    fn available(&self) -> Health {
        Health::Ok
    }
    fn collect(&self) -> Result<Snapshot, CollectError> {
        panic!("a field this build does not know")
    }
}

#[test]
fn a_collector_switched_off_by_a_panic_reads_again_once_its_block_changes() {
    let mut it = watching("following-panicked", Health::Ok, Vec::new());
    let directory = temporary_directory("following-panicked-configuration");
    std::fs::create_dir_all(&directory).expect("a directory");
    let path = directory.join("vigil.yaml");
    written(&path, WATCHING_HOSTS);
    let name = path.to_str().expect("utf-8");
    it.round.followed = Followed::of(
        name,
        Stamp::of(name),
        &load(name).expect("loads"),
        vec![Box::new(FollowingTheFile)],
        &["network"],
    );
    it.round.watches[0].follow(Box::new(PanicsUntilFollowed), RuleSet::of(Vec::new()));
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    assert!(
        it.round.watches[0].switched_off().is_some(),
        "it panicked first"
    );

    written(&path, WATCHING_SUDOERS);
    it.round.follow_the_file(&mut said);

    assert!(it.round.watches[0].switched_off().is_none());
    assert_eq!(
        read_now(&it.round),
        vec![
            "file|/etc/hosts".to_string(),
            "file|/etc/sudoers".to_string()
        ],
        "the changed block is read on the round it was taken up"
    );
    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "and the complaint the panic raised is closed"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_program_silenced_in_the_directory_is_not_reported_from_the_next_round_on_whatever_port_it_opens()
 {
    let mut it = watching(
        "following-program",
        Health::Ok,
        vec![
            snapshot(&[443]),
            snapshot(&[443, 8080]),
            snapshot(&[443, 9090]),
        ],
    );
    let directory = temporary_directory("following-program-configuration");
    std::fs::create_dir_all(directory.join("suppressions")).expect("a directory");
    let path = directory.join("vigil.yaml");
    written(&path, "suppressions_path: suppressions\n");
    let name = path.to_str().expect("utf-8");
    it.round.silences = crate::config::Silences::of(name, &load(name).expect("loads"));
    let mut said = Said::about([("network", "ok".to_string())]);

    written(
        &directory.join("suppressions").join("console.yaml"),
        "suppressions:\n  - program: nginx\n    finding_key_match: \"port.listen|tcp|*\"\n    reason: the proxy moves its ports\n",
    );
    it.round.follow_the_file(&mut said);

    let silence = it.round.shared.with(|state| state.agent().silence);
    assert_eq!(
        silence.suppressions,
        vec!["program nginx · port.listen|tcp|* — the proxy moves its ports".to_string()],
        "the status names the program, so a quiet screen is explained on the screen"
    );
    it.round.read(0, &mut said);
    it.round.read(0, &mut said);
    it.round.read(0, &mut said);
    assert!(
        about_the_host(&it.round).is_empty(),
        "every port the program opened or closed after the entry was written is covered: {:?}",
        about_the_host(&it.round)
    );
    assert!(it.round.policy.suppressed() >= 2);
}
