use std::fs;

use vigil_collect::{Collector, Health, name_of_user, outside_the_sample, this_account};
use vigil_model::{Golden, Shape, Snapshot};

use super::ProcessesCollector;
use super::health::shown_to;
use super::{running, still_running};

fn at_noon() -> String {
    "2026-09-19T12:00:00.000Z".to_string()
}

fn read() -> Snapshot {
    ProcessesCollector::new(at_noon)
        .collect()
        .expect("the table of processes is readable on macOS")
}

fn this_program() -> String {
    let path = std::env::current_exe().expect("the test binary");
    fs::canonicalize(path)
        .expect("canonical")
        .to_string_lossy()
        .into_owned()
}

fn this_row() -> String {
    let me = this_account();
    let name = name_of_user(me).unwrap_or_else(|| me.to_string());
    format!("exec|{}|{name}", this_program())
}

#[test]
fn reads_this_host_and_finds_the_program_of_this_test_under_its_own_account() {
    let snapshot = read();

    assert_eq!(snapshot.source, "processes");
    assert_eq!(snapshot.taken_at, "2026-09-19T12:00:00.000Z");
    let row = snapshot
        .items
        .get(&this_row())
        .unwrap_or_else(|| panic!("{} is running", this_row()));
    assert_eq!(row["exe_deleted"], false);
    assert_eq!(row["uid"], this_account());
    assert!(
        snapshot
            .items
            .keys()
            .any(|key| key.starts_with("exec|/sbin/launchd|")),
        "launchd runs on every Mac, and its program is named to every account"
    );
}

#[test]
fn the_command_line_of_a_process_of_this_account_is_read_and_redacted_like_on_linux() {
    let snapshot = read();
    let row = &snapshot.items[&this_row()];

    assert!(
        row["cmdline"].as_str().is_some() || row["cmdline_varies"] == true,
        "{row}"
    );
}

#[test]
fn a_reading_of_macos_has_the_shape_of_the_reading_the_rules_and_the_console_were_built_on() {
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("processes")
            .held()
            .expect("the published shape of processes"),
    )
    .expect("a shape");

    let drift = outside_the_sample(&Shape::of(&read()), &sample, &[]);

    assert!(drift.is_empty(), "{drift:#?}");
}

#[test]
fn two_readings_a_moment_apart_describe_the_same_host() {
    let first = read();
    let second = read();

    let changed: Vec<&String> = first
        .items
        .keys()
        .chain(second.items.keys())
        .filter(|key| first.items.get(*key) != second.items.get(*key))
        .collect();
    assert!(
        changed.len() <= 4,
        "two readings a moment apart differed in {} rows: {changed:?}",
        changed.len()
    );
}

#[test]
fn an_agent_not_running_as_root_says_so_and_says_whose_processes_it_cannot_read() {
    let health = ProcessesCollector::new(at_noon).available();

    match this_account() {
        0 => assert_eq!(health, Health::Ok),
        me => {
            let Health::Degraded(why) = health else {
                panic!("{health:?}")
            };
            assert!(why.contains("run as root"), "{why}");
            assert!(why.contains(&format!("uid {me}")), "{why}");
        }
    }
}

#[test]
fn the_words_of_an_agent_that_cannot_read_every_process_name_the_account_it_runs_as() {
    assert_eq!(shown_to(0, true), Health::Ok);
    assert_eq!(
        shown_to(501, false),
        Health::Ok,
        "a host where every process is this agent's own hides nothing from it"
    );
    let Health::Degraded(why) = shown_to(3_999_999_999, true) else {
        panic!("degraded")
    };
    assert_eq!(
        why,
        "the command lines of the processes of every account other than uid 3999999999 are not \
         read, and a program of theirs whose file was deleted is not named: macOS shows the \
         arguments of a process only to its own account and to root; run as root",
        "the words stay the same from one reading to the next: a number of processes in them \
         would move on every reading, and every move is a finding about the agent"
    );
}

#[test]
fn this_test_is_found_among_the_processes_running_its_own_program_as_its_own_account() {
    let pids = running(&this_program(), this_account()).expect("the table is read");

    assert!(pids.contains(&std::process::id()), "{pids:?}");
}

#[test]
fn the_same_program_under_another_account_is_not_the_row_that_was_marked() {
    let pids = running(&this_program(), this_account().wrapping_add(7919)).expect("read");

    assert!(!pids.contains(&std::process::id()), "{pids:?}");
}

#[test]
fn a_pid_is_asked_again_whether_it_still_runs_the_program_and_the_account_marked() {
    let program = this_program();
    let pid = std::process::id();
    let me = this_account();

    assert!(still_running(pid, &program, Some(me)));
    assert!(still_running(pid, &program, None));
    assert!(!still_running(pid, "/usr/sbin/nginx", Some(me)));
    assert!(!still_running(pid, &program, Some(me.wrapping_add(7919))));
    assert!(!still_running(99_999_999, &program, None));
}
