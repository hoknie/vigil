use std::fs;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::os::unix::net::UnixListener;

use serde_json::Value;
use vigil_collect::{Collector, Health, name_of_user, outside_the_sample, this_account};
use vigil_model::{Golden, Shape, Snapshot};

use super::NetworkCollector;
use super::health::shown_to;

fn read() -> Snapshot {
    NetworkCollector::new(|| "2026-09-19T12:00:00.000Z".to_string())
        .collect()
        .expect("the sockets of this account are readable on macOS")
}

fn this_program() -> String {
    let path = std::env::current_exe().expect("the test binary");
    fs::canonicalize(path)
        .expect("canonical")
        .to_string_lossy()
        .into_owned()
}

fn held_by_this_test(row: &Value) {
    assert_eq!(row["owner_resolved"], true, "{row}");
    assert_eq!(row["process"]["pid"], std::process::id(), "{row}");
    assert_eq!(row["process"]["exe"], this_program(), "{row}");
    assert_eq!(row["process"]["exe_deleted"], false, "{row}");
    assert!(row["process"]["cmdline"].is_string(), "{row}");
    assert_eq!(
        row["user"].as_str(),
        name_of_user(this_account()).as_deref(),
        "{row}"
    );
}

#[test]
fn a_port_this_test_listens_on_is_found_with_the_process_holding_it() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("bound").port();

    let snapshot = read();

    let key = format!("tcp|127.0.0.1:{port}");
    let row = snapshot
        .items
        .get(&key)
        .unwrap_or_else(|| panic!("{key} is listening"));
    assert_eq!(row["protocol"], "tcp");
    assert_eq!(row["address"], "127.0.0.1");
    assert_eq!(row["port"], port);
    assert_eq!(row["uid"], this_account());
    held_by_this_test(row);
}

#[test]
fn a_v6_port_is_found_in_the_v6_table_under_the_address_linux_would_print() {
    let listener = TcpListener::bind("[::1]:0").expect("a v6 port");
    let port = listener.local_addr().expect("bound").port();

    let snapshot = read();

    let row = &snapshot.items[&format!("tcp6|::1:{port}")];
    assert_eq!(row["protocol"], "tcp6");
    held_by_this_test(row);
}

#[test]
fn a_udp_socket_with_no_peer_is_found_and_one_with_a_peer_is_not() {
    let door = UdpSocket::bind("127.0.0.1:0").expect("a udp port");
    let door_port = door.local_addr().expect("bound").port();
    let talking = UdpSocket::bind("127.0.0.1:0").expect("a udp port");
    talking
        .connect(("127.0.0.1", door_port))
        .expect("connected");
    let talking_port = talking.local_addr().expect("bound").port();

    let snapshot = read();

    held_by_this_test(&snapshot.items[&format!("udp|127.0.0.1:{door_port}")]);
    assert!(
        !snapshot
            .items
            .contains_key(&format!("udp|127.0.0.1:{talking_port}"))
    );
}

#[test]
fn a_conversation_accepted_on_a_port_is_not_a_second_door() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("bound").port();
    let client = TcpStream::connect(("127.0.0.1", port)).expect("connected");
    let (_accepted, _) = listener.accept().expect("accepted");
    let client_port = client.local_addr().expect("bound").port();

    let snapshot = read();

    assert!(
        snapshot
            .items
            .contains_key(&format!("tcp|127.0.0.1:{port}"))
    );
    assert!(
        !snapshot
            .items
            .keys()
            .any(|key| key.ends_with(&format!(":{client_port}"))),
        "the client end of a conversation is not a door"
    );
}

#[test]
fn a_unix_socket_this_test_serves_on_is_found_under_the_path_it_was_bound_to() {
    let path = std::env::temp_dir().join(format!("vigil-network-{}.sock", std::process::id()));
    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bound");

    let snapshot = read();
    drop(listener);
    let _ = fs::remove_file(&path);

    let key = format!("unix|{}", path.display());
    let row = snapshot
        .items
        .get(&key)
        .unwrap_or_else(|| panic!("{key} is served"));
    assert_eq!(row["type"], "stream");
    assert_eq!(row["abstract"], false);
    assert!(row.get("port").is_none());
    held_by_this_test(row);
}

#[test]
fn a_reading_of_macos_has_the_shape_of_the_reading_the_rules_and_the_console_were_built_on() {
    let _listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("network")
            .held()
            .expect("the published shape of network"),
    )
    .expect("a shape");

    let drift = outside_the_sample(&Shape::of(&read()), &sample, &[]);

    assert!(drift.is_empty(), "{drift:#?}");
}

#[test]
fn every_row_is_keyed_the_way_the_rules_read_it() {
    let _listener = TcpListener::bind("127.0.0.1:0").expect("a port");

    for (key, item) in &read().items {
        if key.ends_with("|beyond") {
            assert!(item["count"].as_u64().is_some(), "{key} counts nothing");
            continue;
        }
        assert!(item.get("owner_resolved").is_some(), "{key} lost its flag");
        match key.split_once('|') {
            Some(("tcp" | "tcp6" | "udp" | "udp6", rest)) => {
                assert!(rest.contains(':'), "key shape: {key}")
            }
            Some(("unix", rest)) => assert!(!rest.is_empty(), "key shape: {key}"),
            Some(("outbound", rest)) => assert_eq!(rest.split('|').count(), 3, "key shape: {key}"),
            Some(("inbound", rest)) => assert_eq!(rest.split('|').count(), 4, "key shape: {key}"),
            _ => panic!("key shape: {key}"),
        }
    }
}

#[test]
fn an_agent_not_running_as_root_says_so_and_says_whose_sockets_it_cannot_see() {
    let standing = NetworkCollector::new(String::new).standing();

    assert_eq!(
        standing.health,
        Health::Ok,
        "what this account is not shown is a limit of the mode, and the reading itself is whole"
    );
    match this_account() {
        0 => assert_eq!(standing.limit, None),
        me => {
            let why = standing
                .limit
                .expect("another account runs a process here")
                .to_string();
            assert!(why.contains("run as root"), "{why}");
            assert!(why.contains(&format!("uid {me}")), "{why}");
        }
    }
}

#[test]
fn root_is_neither_limited_nor_degraded() {
    assert_eq!(shown_to(0, true), None);
    assert_eq!(shown_to(501, false), None);
}

#[test]
fn an_account_that_sees_other_accounts_is_limited_and_healthy() {
    let limit = shown_to(3_999_999_999, true).expect("the sockets of other accounts are not seen");

    assert_eq!(
        limit.to_string(),
        "the sockets held by the processes of every account other than uid 3999999999 are \
         not seen at all, a port they listen on included: macOS lists the open files of a \
         process only to its own account and to root; run as root",
        "the words stay the same from one reading to the next, and a port of another account \
         is said to be missing rather than shown as closed"
    );
}
