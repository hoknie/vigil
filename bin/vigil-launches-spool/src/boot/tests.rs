use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use vigil_launches::{ABSENT, ESLOGGER_SPOOL, ESLOGGER_STATUS, REFUSED, STOPPED, SpoolerStatus};

use super::run::run;
use crate::cli::Options;

const RECORDED: &str = include_str!(
    "../../../../crates/collectors/vigil-launches/src/parsers/eslogger/tests/exec.ndjson"
);

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "vigil-launches-spool-run-{}-{named}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a directory");
    at
}

fn options(at: &Path, configuration: &str) -> Options {
    let written = at.join("vigil.yaml");
    fs::write(&written, configuration).expect("a configuration");
    Options {
        directory: at.join("launches").display().to_string(),
        configuration: written.display().to_string(),
        ceiling: 1024 * 1024,
    }
}

fn program(at: &Path, body: &str) -> String {
    let path = at.join("eslogger");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("a script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    path.display().to_string()
}

fn status(at: &Path) -> SpoolerStatus {
    SpoolerStatus::read(&at.join("launches").join(ESLOGGER_STATUS)).expect("a status")
}

fn spool(at: &Path) -> String {
    fs::read_to_string(at.join("launches").join(ESLOGGER_SPOOL)).expect("a spool")
}

#[test]
fn what_eslogger_printed_is_spooled_as_one_record_pair_per_launch_a_person_made() {
    let at = workspace("printed");
    fs::write(at.join("recorded.ndjson"), RECORDED).expect("recorded events");
    let eslogger = program(
        &at,
        &format!("exec /bin/cat {}", at.join("recorded.ndjson").display()),
    );

    run(
        &options(&at, "launches:\n  record_arguments: false\n"),
        &eslogger,
    );

    let written = spool(&at);
    assert_eq!(
        written
            .lines()
            .filter(|line| line.starts_with("type=SYSCALL"))
            .count(),
        3,
        "four launches were printed and launchd started one of them for nobody: {written}"
    );
    assert!(written.contains("\"/usr/bin/curl\""), "{written}");
    assert!(!written.contains("xpcproxy"), "{written}");
    assert!(
        !written.contains("a0="),
        "arguments were not asked for: {written}"
    );
    assert!(
        !written.contains("wJalrXUtnFEMI"),
        "the environment is never written: {written}"
    );
    assert_eq!(status(&at).state, STOPPED);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn arguments_asked_for_are_spooled_with_their_secrets_already_hidden() {
    let at = workspace("arguments");
    fs::write(at.join("recorded.ndjson"), RECORDED).expect("recorded events");
    let eslogger = program(
        &at,
        &format!("exec /bin/cat {}", at.join("recorded.ndjson").display()),
    );

    run(
        &options(&at, "launches:\n  record_arguments: true\n"),
        &eslogger,
    );

    let written = spool(&at);
    assert!(written.contains("redacted=yes"), "{written}");
    assert!(!written.contains("9f8e7d6c"), "{written}");
    assert!(status(&at).arguments_recorded);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn eslogger_that_refuses_to_start_leaves_a_status_saying_what_it_said() {
    let at = workspace("refused");
    let eslogger = program(
        &at,
        "echo 'Failed to create ES client: Not privileged to create an ES client, need to be superuser (ES_NEW_CLIENT_RESULT_ERR_NOT_PRIVILEGED)' >&2\nexit 71",
    );

    run(&options(&at, ""), &eslogger);

    let said = status(&at);
    assert_eq!(said.state, REFUSED);
    assert!(
        said.why
            .as_deref()
            .is_some_and(|why| why.contains("ERR_NOT_PRIVILEGED")),
        "{said:?}"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_mac_without_eslogger_leaves_a_status_saying_it_is_not_there() {
    let at = workspace("absent");

    run(&options(&at, ""), "/usr/bin/no-such-eslogger");

    assert_eq!(status(&at).state, ABSENT);
    let _ = fs::remove_dir_all(&at);
}

#[cfg(target_os = "macos")]
#[test]
fn the_eslogger_of_this_mac_run_by_an_account_that_is_not_root_is_refused_and_says_why() {
    use std::os::unix::fs::MetadataExt;

    let at = workspace("this-mac");
    let me = fs::metadata(&at).expect("stat").uid();
    if me == 0 || !Path::new(vigil_launches::ESLOGGER).exists() {
        let _ = fs::remove_dir_all(&at);
        return;
    }

    run(&options(&at, ""), vigil_launches::ESLOGGER);

    let said = status(&at);
    assert_eq!(said.state, REFUSED, "{said:?}");
    assert!(
        said.why
            .as_deref()
            .is_some_and(|why| why.contains("ES client")),
        "{said:?}"
    );
    let _ = fs::remove_dir_all(&at);
}

#[cfg(target_os = "macos")]
#[test]
fn what_this_program_spools_on_a_mac_is_what_the_collector_of_the_agent_reads() {
    use vigil_collect::{Collector, Health};
    use vigil_launches::LaunchesCollector;

    let at = workspace("read-back");
    fs::write(at.join("recorded.ndjson"), RECORDED).expect("recorded events");
    let eslogger = program(
        &at,
        &format!("exec /bin/cat {}", at.join("recorded.ndjson").display()),
    );
    run(&options(&at, ""), &eslogger);

    let collector = LaunchesCollector::with_paths(
        || "2026-09-19T12:00:00.000Z".to_string(),
        false,
        at.join("launches"),
    );
    let read = collector.collect().expect("the spool is read");

    let programs: Vec<&str> = read
        .items
        .keys()
        .filter(|key| key.starts_with("run|"))
        .filter_map(|key| key.rsplit('|').next())
        .collect();
    assert_eq!(
        programs,
        vec!["/bin/ls", "/private/tmp/.x/beacon", "/usr/bin/curl"]
    );
    assert!(
        matches!(collector.available(), Health::Degraded(ref why) if why.contains("stopped")),
        "the recorded eslogger ended, and a reading that stops receiving says so"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn launches_that_arrive_together_are_written_together_and_none_is_lost() {
    use std::io::BufReader;

    use vigil_launches::SpoolWriter;

    use super::pump::pump;

    let at = workspace("batched");
    let many: String = RECORDED.repeat(200);
    let mut spool = SpoolWriter::open(at.join("spool"), 64 * 1024 * 1024).expect("a spool");
    let mut unwritable = None;

    let pumped = pump(
        &mut BufReader::with_capacity(256 * 1024, many.as_bytes()),
        &mut spool,
        false,
        &mut unwritable,
    );

    assert_eq!(
        pumped.written, 600,
        "three launches a person made, two hundred times"
    );
    assert_eq!(pumped.nobody, 200);
    assert!(
        pumped.writes < 20,
        "every write reaches the disk before it returns, and one per launch is a Mac that \
         compiles something waiting on its own audit: {} writes",
        pumped.writes
    );
    let _ = fs::remove_dir_all(&at);
}
