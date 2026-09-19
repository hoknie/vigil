use std::fs;
use std::path::Path;

use vigil_collect::{Collector, Health, name_of_user, outside_the_sample, this_account};
use vigil_model::{Golden, Shape, Snapshot};

use super::PersistenceCollector;
use super::files::files_in;
use super::places::LAUNCHD_DIRECTORIES;
use crate::parsers::{launchd_facts, parse_plist};

fn read() -> Snapshot {
    PersistenceCollector::new(|| "2026-09-19T12:00:00.000Z".to_string())
        .collect()
        .expect("every Mac has launchd jobs")
}

#[test]
fn every_job_on_the_system_volume_of_this_mac_is_a_property_list_this_build_reads() {
    let mut read = 0usize;
    let mut labelled = 0usize;
    for (directory, _, _) in LAUNCHD_DIRECTORIES {
        for path in files_in(Path::new(directory)) {
            if path.extension().is_none_or(|kind| kind != "plist") {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let job = parse_plist(&bytes)
                .unwrap_or_else(|refusal| panic!("{}: {refusal}", path.display()));
            labelled += usize::from(launchd_facts(&job).label.is_some());
            read += 1;
        }
    }

    assert!(
        labelled > 300,
        "macOS ships hundreds of jobs, in XML and in binary, and {labelled} of the {read} \
         files read carry a label"
    );
}

#[test]
fn a_daemon_of_the_system_volume_is_read_as_the_vendors_and_runs_as_root_unless_it_says_otherwise()
{
    let snapshot = read();
    let (_, job) = snapshot
        .items
        .iter()
        .find(|(key, item)| {
            key.starts_with("launchd|/System/Library/LaunchDaemons/") && item["run_as"] == "root"
        })
        .expect("a daemon of macOS running as root");

    assert_eq!(job["scope"], "vendor");
    assert_eq!(job["domain"], "daemon");
    assert_eq!(job["understood"], true);
}

#[test]
fn the_agents_of_the_person_running_this_test_are_read_as_theirs() {
    let me = name_of_user(this_account()).expect("a name");
    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let agents = Path::new(&home).join("Library/LaunchAgents");
    let listed = files_in(&agents)
        .into_iter()
        .filter(|path| path.extension().is_some_and(|kind| kind == "plist"))
        .count();
    if listed == 0 || !home.starts_with("/Users/") {
        return;
    }

    let snapshot = read();
    let theirs: Vec<&serde_json::Value> = snapshot
        .items
        .iter()
        .filter(|(key, _)| key.starts_with(&format!("launchd|{}/", agents.display())))
        .map(|(_, item)| item)
        .collect();

    assert_eq!(theirs.len(), listed);
    for item in theirs {
        assert_eq!(item["scope"], "person");
        assert_eq!(item["owner"], me.as_str());
    }
}

#[test]
fn the_shell_profiles_macos_ships_are_watched_as_files_other_processes_execute() {
    let snapshot = read();

    let zshrc = &snapshot.items["script|/etc/zshrc"];
    assert_eq!(zshrc["family"], "profile");
    assert_eq!(zshrc["present"], true);
    assert!(zshrc["sha256"].is_string());
}

#[test]
fn a_mac_is_read_with_no_row_of_a_thing_it_does_not_have() {
    let snapshot = read();

    for missing in ["unit|", "timer|", "module|", "modules|", "preload|"] {
        assert!(
            !snapshot.items.keys().any(|key| key.starts_with(missing)),
            "{missing}: a row saying /proc/modules could not be read on a Mac, where there is \
             none to read, is a reading that is always half broken"
        );
    }
}

#[test]
fn an_agent_not_running_as_root_says_which_places_it_cannot_see() {
    let health = PersistenceCollector::new(String::new).available();

    match this_account() {
        0 => assert!(
            matches!(health, Health::Ok | Health::Degraded(_)),
            "{health:?}"
        ),
        _ => {
            let Health::Degraded(why) = health else {
                panic!("{health:?}")
            };
            assert!(why.contains("/usr/lib/cron/tabs"), "{why}");
            assert!(why.contains("login"), "{why}");
            assert!(why.contains("run as root"), "{why}");
        }
    }
}

#[test]
fn a_reading_of_this_mac_has_the_shape_of_the_sample_the_screen_was_built_on() {
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("persistence-macos")
            .held()
            .expect("the published shape of a Mac"),
    )
    .expect("a shape");

    let drift = outside_the_sample(&Shape::of(&read()), &sample, &[]);

    assert!(drift.is_empty(), "{drift:#?}");
}
