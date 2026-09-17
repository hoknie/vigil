use std::time::{Duration, Instant};

use serde_json::{Value, json};
use vigil_collect::{CollectError, Collector, Health};
use vigil_model::Snapshot;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;

use super::harness::{raised, snapshot, temporary_directory, watching};
use crate::config::{Followed, Stamp, load};
use crate::types::Said;

const WATCHING_HOSTS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n";

const WATCHING_SUDOERS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n    - \"/etc/sudoers\"\n";

struct Listed(Value);

impl Collector for Listed {
    fn name(&self) -> &'static str {
        "ports"
    }
    fn available(&self) -> Health {
        Health::Ok
    }
    fn collect(&self) -> Result<Snapshot, CollectError> {
        let mut reading = Snapshot::new("ports", "2026-09-17T12:00:00.000Z");
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
        "ports"
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
            .snapshot("ports")
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
        &["ports"],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
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
        &["ports"],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
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
