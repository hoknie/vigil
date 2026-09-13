use serde_json::{Value, json};
use vigil_model::Change;

use super::verdict::{ports, ports_tick};
use crate::fixture;

#[test]
fn a_new_port_from_a_writable_path_produces_exactly_one_finding() {
    let change = Change::Added {
        key: "tcp|0.0.0.0:4444".into(),
        after: fixture::socket("0.0.0.0", 4444, "/tmp/.x/nc", "www-data"),
    };

    let fired = ports(&change);

    assert_eq!(fired.len(), 1, "fired: {fired:?}");
    assert_eq!(fired[0].0, "listen_from_writable_path");
    assert_eq!(fired[0].1, "port.listen.new");
}

#[test]
fn an_ordinary_new_port_produces_exactly_one_finding_too() {
    let change = Change::Added {
        key: "tcp|0.0.0.0:443".into(),
        after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
    };

    let fired = ports(&change);

    assert_eq!(fired.len(), 1, "fired: {fired:?}");
    assert_eq!(fired[0].0, "new_listening_port");
}

#[test]
fn a_change_no_rule_has_anything_to_say_about_produces_nothing() {
    let mut after = fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root");
    after["uid"] = Value::from(1000);
    let change = Change::Changed {
        key: "tcp|0.0.0.0:443".into(),
        before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
        after,
    };

    assert!(ports(&change).is_empty());
}

#[test]
fn exactly_one_socket_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Vec<Change>, &str, &str)> = vec![
        (
            "a service was deployed",
            vec![Change::Added {
                key: "tcp|0.0.0.0:443".into(),
                after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
            }],
            "new_listening_port",
            "port.listen.new",
        ),
        (
            "something in /tmp opened a port",
            vec![Change::Added {
                key: "tcp|0.0.0.0:4444".into(),
                after: fixture::socket("0.0.0.0", 4444, "/tmp/.x/nc", "www-data"),
            }],
            "listen_from_writable_path",
            "port.listen.new",
        ),
        (
            "the service stopped",
            vec![Change::Removed {
                key: "tcp|0.0.0.0:443".into(),
                before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
            }],
            "closed_listening_port",
            "port.listen.removed",
        ),
        (
            "another binary answers on the same port",
            vec![Change::Changed {
                key: "tcp|0.0.0.0:443".into(),
                before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
                after: fixture::socket("0.0.0.0", 443, "/usr/local/bin/caddy", "root"),
            }],
            "listening_port_owner_changed",
            "port.listen.owner_changed",
        ),
        (
            "the binary behind a port was unlinked",
            vec![Change::Changed {
                key: "tcp|0.0.0.0:5555".into(),
                before: fixture::socket("0.0.0.0", 5555, "/opt/app/server", "www-data"),
                after: fixture::socket_with_deleted_binary("0.0.0.0", 5555, "/opt/app/server"),
            }],
            "listening_binary_deleted",
            "process.binary_deleted",
        ),
        (
            "`listen 127.0.0.1:5432` became `listen 5432` — two changes, one event",
            vec![
                Change::Added {
                    key: "tcp|0.0.0.0:5432".into(),
                    after: fixture::socket("0.0.0.0", 5432, "/usr/bin/postgres", "postgres"),
                },
                Change::Removed {
                    key: "tcp|127.0.0.1:5432".into(),
                    before: fixture::socket("127.0.0.1", 5432, "/usr/bin/postgres", "postgres"),
                },
            ],
            "exposed_listening_port",
            "port.listen.exposed",
        ),
        (
            "a binary in /dev/shm holding a unix socket",
            vec![Change::Added {
                key: "unix|/tmp/.s.sock".into(),
                after: fixture::unix_socket("/tmp/.s.sock", "/dev/shm/payload", "www-data"),
            }],
            "listen_from_writable_path",
            "port.listen.new",
        ),
        (
            "a unix socket changed hands",
            vec![Change::Changed {
                key: "unix|/run/mysqld/mysqld.sock".into(),
                before: fixture::unix_socket(
                    "/run/mysqld/mysqld.sock",
                    "/usr/sbin/mysqld",
                    "mysql",
                ),
                after: fixture::unix_socket("/run/mysqld/mysqld.sock", "/tmp/mysqld", "mysql"),
            }],
            "listening_port_owner_changed",
            "port.listen.owner_changed",
        ),
    ];

    for (what, changes, rule, kind) in cases {
        let fired = ports_tick(&changes);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_unix_socket_coming_and_going_is_carried_in_the_snapshot_and_reported_by_nobody() {
    let appeared = Change::Added {
        key: "unix|/run/containerd/s/338de3703dd8".into(),
        after: fixture::unix_socket(
            "/run/containerd/s/338de3703dd8",
            "/usr/bin/containerd-shim",
            "root",
        ),
    };
    let went = Change::Removed {
        key: "unix|/run/containerd/s/338de3703dd8".into(),
        before: fixture::unix_socket(
            "/run/containerd/s/338de3703dd8",
            "/usr/bin/containerd-shim",
            "root",
        ),
    };

    assert!(ports(&appeared).is_empty());
    assert!(ports(&went).is_empty());
}

#[test]
fn the_row_counting_the_unix_sockets_with_no_name_reaches_no_rule_at_all() {
    let change = Change::Added {
        key: "unix|unnamed".into(),
        after: json!({"protocol": "unix", "count": 3, "owner_resolved": false}),
    };

    assert!(ports(&change).is_empty());
}
