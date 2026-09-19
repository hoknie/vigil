use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::{
    ESLOGGER_SPOOL, ESLOGGER_STATUS, LAUNCHES_DIRECTORY, REFUSED, SpoolWriter, SpoolerStatus,
    audit_records, somebody_launched,
};
use crate::parsers::{Launched, parse_audit_log, record_is_read};

fn curl() -> Launched {
    Launched {
        seconds: 1_789_808_523,
        milliseconds: 412,
        sequence: 3421,
        pid: 2170,
        ppid: 2143,
        auid: 501,
        uid: 501,
        euid: 501,
        executable: "/usr/bin/curl".to_string(),
        arguments: vec![
            "curl".to_string(),
            "-H".to_string(),
            "Authorization: Bearer 9f8e7d6c".to_string(),
            "https://updates.example.net/v2".to_string(),
        ],
        working_directory: Some("/Users/alice".to_string()),
    }
}

fn temporary(name: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!("vigil-spool-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a directory");
    at
}

#[test]
fn a_launch_eslogger_saw_is_read_back_by_the_parser_the_audit_spool_is_read_with() {
    let written = audit_records(&curl(), true);
    let read = parse_audit_log(written.as_bytes(), true);

    assert_eq!(read.executions.len(), 1, "{written}");
    let execution = &read.executions[0];
    assert_eq!(execution.id, "1789808523.412:3421");
    assert_eq!(execution.auid, Some(501));
    assert_eq!(execution.executable.as_deref(), Some("/usr/bin/curl"));
    assert!(
        execution.arguments_redacted,
        "a spool that hid a header says it hid one, so a reader sees hidden and not empty"
    );
}

#[test]
fn a_secret_in_the_arguments_is_hidden_before_the_record_is_written() {
    let written = audit_records(&curl(), true);

    assert!(!written.contains("9f8e7d6c"), "{written}");
    assert!(
        !written.contains(&hex("9f8e7d6c")),
        "hex is how a record carries a space, and it is not a hiding place: {written}"
    );
}

#[test]
fn with_arguments_off_not_one_argument_reaches_the_spool() {
    let written = audit_records(&curl(), false);

    assert!(!written.contains("a0="), "{written}");
    assert!(!written.contains(&hex("updates.example.net")), "{written}");
    assert!(!written.contains("updates.example.net"), "{written}");
    assert!(
        written.contains("argc=4"),
        "how many there were is kept, what they were is not: {written}"
    );
}

#[test]
fn a_program_whose_path_holds_a_space_or_a_quote_survives_the_record() {
    let mut odd = curl();
    odd.executable = "/Applications/Some \"App\".app/Contents/MacOS/Some App".to_string();

    let read = parse_audit_log(audit_records(&odd, false).as_bytes(), false);

    assert_eq!(
        read.executions[0].executable.as_deref(),
        Some(odd.executable.as_str())
    );
}

#[test]
fn a_launch_no_person_logged_in_for_is_not_written_down() {
    let mut daemon = curl();
    daemon.auid = u32::MAX;

    assert!(somebody_launched(&curl()));
    assert!(
        !somebody_launched(&daemon),
        "a Mac starts thousands of daemons an hour with no audit user, and the reading of \
         what people run drops every one of them; the spool is not the place to keep them"
    );
}

#[test]
fn every_line_of_a_record_is_one_the_spool_writer_keeps() {
    let written = audit_records(&curl(), true);

    for line in written.lines() {
        assert!(record_is_read(line.as_bytes()), "{line}");
    }

    let at = temporary("kept");
    let spool = at.join(ESLOGGER_SPOOL);
    let mut writer = SpoolWriter::open(&spool, 1024 * 1024).expect("opens");
    writer.write(written.as_bytes()).expect("writes");

    assert_eq!(writer.report().kept, 2);
    assert_eq!(fs::read_to_string(&spool).expect("reads"), written);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn the_status_of_eslogger_is_written_whole_for_the_owner_alone_and_read_back() {
    let at = temporary("status");
    let path = at.join(ESLOGGER_STATUS);
    let status = SpoolerStatus {
        state: REFUSED.to_string(),
        since: "2026-09-19T09:02:03.000Z".to_string(),
        program: "/usr/bin/eslogger".to_string(),
        arguments_recorded: false,
        why: Some(
            "Not permitted to create an ES client (ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED)"
                .to_string(),
        ),
    };

    status.write(&path).expect("writes");

    assert_eq!(SpoolerStatus::read(&path).expect("reads"), status);
    let mode = fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn on_a_mac_the_spool_lives_under_the_state_directory_the_package_makes() {
    assert_eq!(
        LAUNCHES_DIRECTORY,
        format!(
            "{}/launches",
            vigil_config::Installation::MACOS.state_directory
        )
    );
}

fn hex(text: &str) -> String {
    text.bytes().map(|byte| format!("{byte:02X}")).collect()
}
