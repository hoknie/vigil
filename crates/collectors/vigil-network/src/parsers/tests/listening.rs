use super::super::listening::{ProcessOwner, SocketsReading, listening_snapshot};
use super::super::proc_net::{Protocol, SocketRow};
use super::super::proc_net_unix::{UnixKind, UnixSocketRow};
use serde_json::Value;
use std::collections::BTreeMap;

fn row(address: &str, port: u16, uid: u32, inode: u64) -> SocketRow {
    SocketRow {
        protocol: Protocol::Tcp,
        address: address.to_string(),
        port,
        uid,
        inode,
    }
}

fn unix(name: &str, kind: UnixKind, inode: u64) -> UnixSocketRow {
    UnixSocketRow {
        kind,
        name: name.to_string(),
        is_abstract: name.starts_with('@'),
        inode,
    }
}

fn reading<'a>(
    network: &'a [SocketRow],
    unix: &'a [UnixSocketRow],
    owners: &'a BTreeMap<u64, ProcessOwner>,
    users: &'a BTreeMap<u32, String>,
) -> SocketsReading<'a> {
    SocketsReading {
        network,
        unix,
        unnamed_unix: 0,
        owners,
        users,
    }
}

#[test]
fn keys_a_socket_by_protocol_address_and_port() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(
            &[row("0.0.0.0", 4444, 33, 20481)],
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
        ),
    );

    assert_eq!(snapshot.source, "network");
    assert!(snapshot.items.contains_key("tcp|0.0.0.0:4444"));
}

#[test]
fn keys_a_unix_socket_by_the_name_a_person_would_type() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(
            &[],
            &[
                unix("/run/docker.sock", UnixKind::Stream, 700),
                unix("@/tmp/.X11-unix/X0", UnixKind::Stream, 701),
            ],
            &BTreeMap::new(),
            &BTreeMap::new(),
        ),
    );

    assert!(snapshot.items.contains_key("unix|/run/docker.sock"));
    let abstract_socket = &snapshot.items["unix|@/tmp/.X11-unix/X0"];
    assert_eq!(abstract_socket["abstract"], true);
    assert_eq!(abstract_socket["path"], "@/tmp/.X11-unix/X0");
    assert_eq!(abstract_socket["port"], Value::Null, "there is no port");
}

#[test]
fn a_unix_socket_takes_its_user_from_the_process_because_the_kernel_names_none() {
    let owners = BTreeMap::from([(
        700,
        ProcessOwner {
            pid: Some(812),
            executable: Some("/usr/bin/dockerd".into()),
            uid: Some(0),
            ..ProcessOwner::default()
        },
    )]);
    let users = BTreeMap::from([(0, "root".to_string())]);

    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(
            &[],
            &[
                unix("/run/docker.sock", UnixKind::Stream, 700),
                unix("/run/orphan.sock", UnixKind::Stream, 701),
            ],
            &owners,
            &users,
        ),
    );

    assert_eq!(snapshot.items["unix|/run/docker.sock"]["user"], "root");
    let orphan = &snapshot.items["unix|/run/orphan.sock"];
    assert_eq!(
        orphan["user"],
        Value::Null,
        "'we could not look' must not be written as 'root'"
    );
    assert_eq!(orphan["owner_resolved"], false);
}

#[test]
fn sockets_listening_under_no_name_are_counted_rather_than_dropped() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &SocketsReading {
            network: &[],
            unix: &[],
            unnamed_unix: 3,
            owners: &BTreeMap::new(),
            users: &BTreeMap::new(),
        },
    );

    assert_eq!(snapshot.items["unix|unnamed"]["count"], 3);
}

#[test]
fn no_such_sockets_means_no_such_item_rather_than_a_zero() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(&[], &[], &BTreeMap::new(), &BTreeMap::new()),
    );

    assert!(
        !snapshot.items.contains_key("unix|unnamed"),
        "an item that appears on every host to say 'zero' is one nobody reads"
    );
}

#[test]
fn an_unresolved_owner_is_marked_unresolved_rather_than_left_looking_empty() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(
            &[row("0.0.0.0", 22, 0, 20481)],
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
        ),
    );

    let item = &snapshot.items["tcp|0.0.0.0:22"];
    assert_eq!(item["process"], Value::Null);
    assert_eq!(
        item["owner_resolved"], false,
        "'we could not look' and 'nothing owns it' must not be the same document"
    );
    assert_eq!(
        item["uid"], 0,
        "the kernel told us the owner uid regardless"
    );
}

#[test]
fn carries_the_executable_the_user_and_the_redaction_flag() {
    let owners = BTreeMap::from([(
        20481,
        ProcessOwner {
            pid: Some(30211),
            executable: Some("/tmp/.x/nc".into()),
            executable_deleted: true,
            command_line: Some("nc -l -p 4444 -e [redacted]".into()),
            command_line_redacted: true,
            uid: Some(33),
        },
    )]);
    let users = BTreeMap::from([(33, "www-data".to_string())]);

    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(&[row("0.0.0.0", 4444, 33, 20481)], &[], &owners, &users),
    );

    let item = &snapshot.items["tcp|0.0.0.0:4444"];
    assert_eq!(item["user"], "www-data");
    assert_eq!(item["process"]["pid"], 30211);
    assert_eq!(item["process"]["exe"], "/tmp/.x/nc");
    assert_eq!(item["process"]["exe_deleted"], true);
    assert_eq!(item["process"]["cmdline_redacted"], true);
    assert_eq!(item["owner_resolved"], true);
}

#[test]
fn the_pid_of_a_socket_nobody_could_look_behind_is_absent_rather_than_zero() {
    let snapshot = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(
            &[row("0.0.0.0", 22, 0, 20481)],
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
        ),
    );

    assert_eq!(
        snapshot.items["tcp|0.0.0.0:22"]["process"],
        Value::Null,
        "pid 0 is the kernel scheduler, and an operator reading it off this screen and \
         typing it into kill has been handed a lie by the console"
    );
}

#[test]
fn the_same_reading_twice_produces_the_same_document() {
    let owners = BTreeMap::from([(20481, ProcessOwner::default())]);
    let rows = [
        row("0.0.0.0", 22, 0, 20481),
        row("127.0.0.1", 5432, 106, 23901),
    ];
    let unix_rows = [unix("/run/docker.sock", UnixKind::Stream, 700)];
    let users = BTreeMap::new();

    let first = listening_snapshot(
        "2026-09-08T12:00:00.000Z",
        &reading(&rows, &unix_rows, &owners, &users),
    );
    let second = listening_snapshot(
        "2026-09-08T12:00:30.000Z",
        &reading(&rows, &unix_rows, &owners, &users),
    );

    assert_eq!(first.items, second.items);
}
