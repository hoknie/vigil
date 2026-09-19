use super::super::{EsloggerRefusal, Launched, parse_eslogger_event};

const EXECS: &str = include_str!("exec.ndjson");

const FORK: &str = include_str!("fork.json");

fn read() -> Vec<Launched> {
    EXECS
        .lines()
        .map(|line| parse_eslogger_event(line.as_bytes()).expect("an exec eslogger printed"))
        .collect()
}

#[test]
fn an_exec_event_names_the_program_of_the_new_image_and_not_the_one_that_called_exec_first() {
    let launched = &read()[0];

    assert_eq!(launched.executable, "/usr/bin/curl");
    assert_eq!(launched.pid, 2170);
    assert_eq!(launched.ppid, 2143);
    assert_eq!(
        launched.arguments,
        vec![
            "curl",
            "-H",
            "Authorization: Bearer 9f8e7d6c",
            "https://updates.example.net/v2"
        ]
    );
    assert_eq!(launched.working_directory.as_deref(), Some("/Users/alice"));
}

#[test]
fn the_person_behind_a_launch_is_the_audit_user_and_survives_sudo() {
    let as_root = &read()[2];

    assert_eq!(as_root.executable, "/bin/ls");
    assert_eq!(as_root.auid, 501, "alice logged in, and alice ran it");
    assert_eq!(as_root.uid, 0);
    assert_eq!(as_root.euid, 0);
}

#[test]
fn a_daemon_launchd_started_carries_the_audit_user_nobody_logged_in_as() {
    assert_eq!(
        read()[1].auid,
        u32::MAX,
        "launchd starts its daemons with the audit user unset, as the kernel of Linux does"
    );
}

#[test]
fn the_moment_and_the_sequence_eslogger_gave_an_event_become_its_identity() {
    let first = &read()[0];

    assert_eq!(first.seconds, 1_789_808_523);
    assert_eq!(first.milliseconds, 412);
    assert_eq!(
        first.sequence, 3421,
        "the global sequence counts every event eslogger delivered, across event types"
    );
}

#[test]
fn a_time_written_with_fewer_digits_is_still_the_same_moment() {
    let sudo = &read()[2];

    assert_eq!(sudo.seconds, 1_789_808_525);
    assert_eq!(sudo.milliseconds, 900);
}

#[test]
fn the_environment_of_a_launch_is_never_carried_out_of_the_parser() {
    let printed = format!("{:?}", read());

    assert!(
        !printed.contains("AWS_SECRET_ACCESS_KEY") && !printed.contains("wJalrXUtnFEMI"),
        "eslogger prints the environment of every exec, and the environment is where \
         credentials live"
    );
}

#[test]
fn an_event_of_another_type_is_refused_as_not_an_exec_event() {
    assert_eq!(
        parse_eslogger_event(FORK.trim_end().as_bytes()),
        Err(EsloggerRefusal::NotAnExec)
    );
}

#[test]
fn a_line_that_is_not_json_is_refused_rather_than_read_as_nothing() {
    for rubbish in [
        &b"Failed to create ES client: Not privileged"[..],
        b"{\"event\": ",
        b"",
    ] {
        assert_eq!(
            parse_eslogger_event(rubbish),
            Err(EsloggerRefusal::NotJson),
            "{:?}",
            String::from_utf8_lossy(rubbish)
        );
    }
}

#[test]
fn an_exec_without_a_person_or_a_program_is_refused_by_the_field_it_lacks() {
    let mut event: serde_json::Value =
        serde_json::from_str(EXECS.lines().next().expect("a line")).expect("json");
    event["event"]["exec"]["target"]["audit_token"]
        .as_object_mut()
        .expect("a token")
        .remove("auid");

    assert_eq!(
        parse_eslogger_event(event.to_string().as_bytes()),
        Err(EsloggerRefusal::Missing("auid")),
        "a launch with no audit user would be counted against uid 0"
    );
}
