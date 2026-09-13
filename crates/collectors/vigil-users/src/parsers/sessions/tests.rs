use super::utmp::{LIVE, record};
use super::*;

const LOGIND_SSH: &str = "\
# This is private data. Do not parse.
UID=1000
USER=deploy
ACTIVE=1
STATE=active
REMOTE=1
REMOTE_HOST=10.0.0.7
SERVICE=sshd
SCOPE=session-83.scope
LEADER=4021
TYPE=tty
CLASS=user
TTY=pts/0
";

const LOGIND_BACKGROUND: &str = "\
UID=1000
USER=deploy
STATE=closing
REMOTE=0
SERVICE=systemd-user
SCOPE=session-84.scope
LEADER=4100
TYPE=unspecified
CLASS=background
";

#[test]
fn reads_the_live_logins_and_leaves_the_tombstones_alone() {
    let mut file = record(LIVE, 4021, "pts/0", "deploy", "10.0.0.7");
    file.extend(record(LIVE, 900, "tty1", "root", ""));
    file.extend(record(8, 3311, "pts/1", "alice", "10.0.0.9"));
    file.extend(record(2, 0, "~", "reboot", "6.6.0"));

    let sessions = parse_utmp(&file).expect("a utmp file");

    assert_eq!(sessions.len(), 2, "{sessions:?}");
    assert_eq!(sessions[0].user, "deploy");
    assert_eq!(sessions[0].line, "pts/0");
    assert_eq!(sessions[0].from, "10.0.0.7");
    assert_eq!(sessions[0].pid, 4021);
    assert!(sessions[0].is_remote());
    assert_eq!(sessions[0].seen_by(), vec![UTMP]);
    assert!(
        !sessions[1].is_remote(),
        "a console login has no source address"
    );
}

#[test]
fn a_file_in_a_shape_we_do_not_know_is_not_an_empty_host() {
    assert!(parse_utmp(&[0u8; 100]).is_none());
    assert!(parse_utmp(b"not a utmp file").is_none());
    assert_eq!(parse_utmp(&[]), Some(Vec::new()));
}

#[test]
fn a_logind_session_file_gives_the_person_the_terminal_and_where_they_came_from() {
    let session = parse_logind_session("83", LOGIND_SSH).expect("a session file");

    assert_eq!(session.user, "deploy");
    assert_eq!(session.uid, Some(1000));
    assert_eq!(session.line, "pts/0");
    assert_eq!(session.from, "10.0.0.7");
    assert!(session.is_remote());
    assert_eq!(session.pid, 4021);
    assert_eq!(session.id, "83");
    assert_eq!(session.service, "sshd");
    assert_eq!(session.kind, "tty");
    assert_eq!(session.class, "user");
    assert_eq!(session.state, "active");
    assert_eq!(session.seen_by(), vec![LOGIND]);
    assert!(session.attended());
}

#[test]
fn the_other_names_in_that_directory_are_not_sessions_and_do_not_stop_the_reading() {
    assert!(is_session_file("83"));
    assert!(is_session_file("c1"));
    assert!(!is_session_file("83.ref"));
    assert!(!is_session_file(".#session83abcd"));
    assert!(!is_session_file(""));

    assert!(
        parse_logind_session("83", "# This is private data. Do not parse.\n").is_none(),
        "a file holding nothing we understand is not a person logged in"
    );
    assert!(parse_logind_session("83", "").is_none());
}

#[test]
fn a_session_file_that_names_only_the_number_of_the_person_still_counts_as_a_session() {
    let session = parse_logind_session("7", "UID=1000\nLEADER=4021\nTTY=pts/2\n").expect("read");

    assert_eq!(session.user, "");
    assert_eq!(session.who(), "uid 1000");
    assert_eq!(session.key(), "session|uid 1000|pts/2");
}

#[test]
fn one_login_seen_by_two_sources_is_one_row_that_names_them_both() {
    let mut from_utmp =
        parse_utmp(&record(LIVE, 4021, "pts/0", "deploy", "10.0.0.7")).expect("a utmp file");
    from_utmp.push(parse_logind_session("83", LOGIND_SSH).expect("a session file"));

    let merged = merge_sessions(from_utmp);

    assert_eq!(merged.len(), 1, "{merged:?}");
    assert_eq!(merged[0].key(), "session|deploy|pts/0");
    assert_eq!(merged[0].seen_by(), vec![LOGIND, UTMP]);
    assert_eq!(
        merged[0].service, "sshd",
        "logind filled in what utmp lacks"
    );
    assert_eq!(merged[0].id, "83");
}

#[test]
fn a_login_only_one_source_saw_says_which_one_saw_it() {
    let merged = merge_sessions(vec![
        parse_logind_session("83", LOGIND_SSH).expect("a session file"),
    ]);

    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].seen_by(),
        vec![LOGIND],
        "logind knows and utmp does not: that is a fact about the host, not a detail"
    );
}

#[test]
fn a_session_with_no_terminal_keeps_a_row_of_its_own_under_the_number_logind_gave_it() {
    let merged = merge_sessions(vec![
        parse_logind_session("83", LOGIND_SSH).expect("read"),
        parse_logind_session("84", LOGIND_BACKGROUND).expect("read"),
    ]);

    assert_eq!(merged.len(), 2, "{merged:?}");
    let keys: Vec<String> = merged.iter().map(Session::key).collect();
    assert_eq!(
        keys,
        vec!["session|deploy|logind:84", "session|deploy|pts/0"]
    );
}

#[test]
fn a_session_without_a_terminal_joins_the_row_its_leader_already_holds() {
    let mut records = parse_utmp(&record(LIVE, 4021, "pts/0", "deploy", "10.0.0.7")).expect("read");
    records.push(
        parse_logind_session("83", "UID=1000\nUSER=deploy\nLEADER=4021\nCLASS=user\n")
            .expect("read"),
    );

    let merged = merge_sessions(records);

    assert_eq!(merged.len(), 1, "{merged:?}");
    assert_eq!(merged[0].key(), "session|deploy|pts/0");
    assert_eq!(merged[0].seen_by(), vec![LOGIND, UTMP]);
}

#[test]
fn the_key_of_a_session_is_the_same_in_the_next_reading_of_it() {
    let first = merge_sessions(vec![parse_logind_session("83", LOGIND_SSH).expect("read")]);
    let again = merge_sessions(vec![parse_logind_session("83", LOGIND_SSH).expect("read")]);

    assert_eq!(first[0].key(), again[0].key());
    assert_eq!(
        first[0].key(),
        "session|deploy|pts/0",
        "the key is what a person copies into a suppression: it says who and where"
    );
}

#[test]
fn merging_two_sources_puts_the_rows_in_the_same_order_whichever_was_read_first() {
    let one = parse_logind_session("83", LOGIND_SSH).expect("read");
    let two = parse_logind_session("84", LOGIND_BACKGROUND).expect("read");

    let forwards: Vec<String> = merge_sessions(vec![one.clone(), two.clone()])
        .iter()
        .map(Session::key)
        .collect();
    let backwards: Vec<String> = merge_sessions(vec![two, one])
        .iter()
        .map(Session::key)
        .collect();

    assert_eq!(forwards, backwards);
}

#[test]
fn a_closing_session_and_a_background_one_are_not_a_person_at_a_terminal() {
    let background = parse_logind_session("84", LOGIND_BACKGROUND).expect("read");
    assert!(!background.attended());

    let closing =
        parse_logind_session("85", "USER=deploy\nCLASS=user\nSTATE=closing\n").expect("read");
    assert!(!closing.attended());

    let manager =
        parse_logind_session("86", "USER=deploy\nCLASS=manager\nSTATE=active\n").expect("read");
    assert!(!manager.attended());

    let greeter =
        parse_logind_session("87", "USER=gdm\nCLASS=greeter\nSTATE=active\n").expect("read");
    assert!(greeter.attended(), "somebody is at that screen");
}

#[test]
fn a_login_utmp_recorded_is_attended_even_though_it_carries_no_class() {
    let sessions = parse_utmp(&record(LIVE, 900, "tty1", "root", "")).expect("read");

    assert!(sessions[0].attended());
}

#[test]
fn a_source_that_was_not_read_is_not_a_source_that_said_nothing() {
    let absent = SessionSource::absent(UTMP, "/run/utmp", "no file there");
    let refused = SessionSource::refused(LOGIND, "/run/systemd/sessions", "permission denied");
    let empty = SessionSource::read(UTMP, "/run/utmp", 0).saying("the file is empty");
    let quiet = SessionSource::read(LOGIND, "/run/systemd/sessions", 0);

    assert!(!absent.answers());
    assert!(!refused.answers());
    assert!(!empty.answers());
    assert!(
        quiet.answers(),
        "a source that was read and held nothing is a source that works"
    );
    assert_eq!(quiet.sessions, 0);
}
