use std::fs;
use std::path::{Path, PathBuf};

use vigil_collect::{CollectError, Collector, Health, name_of_user, this_account};
use vigil_model::Snapshot;

use super::LaunchesCollector;
use crate::parsers::Launched;
use crate::spool::{
    ESLOGGER_SPOOL, ESLOGGER_STATUS, REFUSED, RUNNING, SpoolWriter, SpoolerStatus, audit_records,
};

const AT: &str = "2026-09-19T12:00:00.000Z";

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "vigil-launches-macos-{}-{named}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a directory of its own");
    at
}

fn status(at: &Path, state: &str, arguments_recorded: bool, why: Option<&str>) {
    SpoolerStatus {
        state: state.to_string(),
        since: "2026-09-19T11:58:00.000Z".to_string(),
        program: "/usr/bin/eslogger".to_string(),
        arguments_recorded,
        why: why.map(str::to_string),
    }
    .write(&at.join(ESLOGGER_STATUS))
    .expect("a status");
}

fn launched(executable: &str, sequence: u64) -> Launched {
    Launched {
        seconds: 1_789_808_523 + sequence,
        milliseconds: 0,
        sequence,
        pid: 4000 + sequence as u32,
        ppid: 1,
        auid: this_account(),
        uid: this_account(),
        euid: this_account(),
        executable: executable.to_string(),
        arguments: vec![
            "run".to_string(),
            "--password".to_string(),
            "hunter2".to_string(),
        ],
        working_directory: None,
    }
}

fn spooled(at: &Path, launches: &[Launched], keep_arguments: bool) {
    let mut writer = SpoolWriter::open(at.join(ESLOGGER_SPOOL), 1024 * 1024).expect("a spool");
    for one in launches {
        writer
            .write(audit_records(one, keep_arguments).as_bytes())
            .expect("written");
    }
}

fn collector(at: &Path, keep_arguments: bool) -> LaunchesCollector {
    LaunchesCollector::with_paths(|| AT.to_string(), keep_arguments, at)
}

fn me() -> String {
    name_of_user(this_account()).expect("this account has a name")
}

fn runs(read: &Snapshot) -> Vec<&str> {
    read.items
        .keys()
        .filter(|key| key.starts_with("run|"))
        .map(String::as_str)
        .collect()
}

#[test]
fn a_mac_where_the_job_has_never_run_says_launches_are_not_visible_and_what_starts_them() {
    let at = workspace("never");
    let collector = collector(&at, false);

    match collector.available() {
        Health::Unavailable(why) => {
            assert!(why.contains("vigil.launches"), "{why}");
            assert!(why.contains("Full Disk Access"), "{why}");
            assert!(why.contains("vigil-launches-spool"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    assert!(
        matches!(collector.collect(), Err(CollectError::Absent(_))),
        "no spool is not a Mac where nobody runs anything"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn eslogger_that_refused_to_start_makes_the_reading_unavailable_with_what_it_said() {
    let at = workspace("refused");
    status(
        &at,
        REFUSED,
        false,
        Some(
            "Failed to create ES client: Not permitted to create an ES client, need to be granted TCC approval (ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED)",
        ),
    );

    match collector(&at, false).available() {
        Health::Unavailable(why) => {
            assert!(why.contains("ERR_NOT_PERMITTED"), "{why}");
            assert!(why.contains("Full Disk Access"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_launch_in_the_spool_is_read_as_the_person_who_logged_in_and_the_program_they_ran() {
    let at = workspace("read");
    status(&at, RUNNING, false, None);
    spooled(&at, &[launched("/usr/bin/curl", 1)], false);

    let collector = collector(&at, false);
    let read = collector.collect().expect("reads");

    assert_eq!(collector.available(), Health::Ok);
    let key = format!("run|{}|/usr/bin/curl", me());
    assert_eq!(runs(&read), vec![key.as_str()]);
    assert_eq!(read.items[&key]["arguments"], serde_json::Value::Null);
    assert_eq!(read.items["launches|source"]["from"], "eslogger");
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_program_under_private_tmp_or_a_home_directory_is_one_from_a_writable_path_on_a_mac() {
    let at = workspace("writable");
    status(&at, RUNNING, false, None);
    spooled(
        &at,
        &[
            launched("/private/tmp/.x/beacon", 1),
            launched("/Users/Shared/agent", 2),
            launched("/usr/bin/curl", 3),
        ],
        false,
    );

    let read = collector(&at, false).collect().expect("reads");
    let writable =
        |program: &str| read.items[&format!("run|{}|{program}", me())]["writable_path"].clone();

    assert_eq!(writable("/private/tmp/.x/beacon"), true);
    assert_eq!(writable("/Users/Shared/agent"), true);
    assert_eq!(writable("/usr/bin/curl"), false);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn arguments_kept_are_the_ones_the_job_hid_its_secrets_in_before_it_wrote_them() {
    let at = workspace("arguments");
    status(&at, RUNNING, true, None);
    spooled(&at, &[launched("/usr/bin/curl", 1)], true);

    let read = collector(&at, true).collect().expect("reads");
    let row = &read.items[&format!("run|{}|/usr/bin/curl", me())];

    assert_eq!(row["arguments_redacted"], true);
    assert!(
        !row["arguments"].as_str().expect("kept").contains("hunter2"),
        "{row}"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn the_next_reading_starts_where_the_last_one_stopped_even_in_a_new_daemon() {
    let at = workspace("resumed");
    status(&at, RUNNING, false, None);
    spooled(&at, &[launched("/usr/bin/curl", 1)], false);

    let first = collector(&at, false).collect().expect("reads");
    spooled(&at, &[launched("/usr/bin/curl", 2)], false);

    let restarted = collector(&at, false);
    restarted.restore(&first);
    let second = restarted.collect().expect("reads");

    assert_eq!(
        second.items[&format!("run|{}|/usr/bin/curl", me())]["runs"],
        2,
        "the first launch was counted before the restart and the second after it; neither \
         is counted twice"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn arguments_asked_for_and_not_spooled_say_the_job_must_read_the_configuration_again() {
    let at = workspace("mismatch");
    status(&at, RUNNING, false, None);
    spooled(&at, &[], false);

    match collector(&at, true).available() {
        Health::Degraded(why) => {
            assert!(why.contains("kickstart -k system/vigil.launches"), "{why}")
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn on_this_mac_the_collector_reads_what_the_launchd_job_writes_or_says_why_it_cannot() {
    let collector = LaunchesCollector::new(|| AT.to_string(), false);

    assert_eq!(
        collector.spool_path,
        PathBuf::from("/usr/local/var/lib/vigil/launches/exec-spool")
    );
    match collector.available() {
        Health::Ok => assert!(collector.collect().is_ok()),
        Health::Degraded(why) | Health::Unavailable(why) => assert!(!why.is_empty()),
    }
}
