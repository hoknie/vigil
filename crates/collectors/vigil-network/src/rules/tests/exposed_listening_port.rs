use std::collections::BTreeSet;

use vigil_model::Finding;

use super::super::exposed_listening_port::ExposedListeningPort;
use crate::fixture;
use vigil_model::{Change, Severity};
use vigil_rules::{BatchRule, RuleContext};

fn claimed_by(changes: &[Change]) -> BTreeSet<String> {
    let mut mint = || "event-1".to_string();
    let mut ctx = RuleContext {
        now: "2026-09-08T12:04:02.104Z".into(),
        mint_event_id: &mut mint,
    };
    ExposedListeningPort.apply(changes, &mut ctx).claimed
}

fn apply(changes: &[Change]) -> Vec<Finding> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-08T12:04:02.104Z".into(),
        mint_event_id: &mut mint,
    };
    ExposedListeningPort.apply(changes, &mut ctx).findings
}

fn moved_off_loopback() -> Vec<Change> {
    vec![
        Change::Added {
            key: "tcp|0.0.0.0:5432".into(),
            after: fixture::socket("0.0.0.0", 5432, "/usr/bin/postgres", "postgres"),
        },
        Change::Removed {
            key: "tcp|127.0.0.1:5432".into(),
            before: fixture::socket("127.0.0.1", 5432, "/usr/bin/postgres", "postgres"),
        },
    ]
}

#[test]
fn a_service_that_leaves_loopback_is_one_finding_with_both_sides_of_the_move() {
    let fired = apply(&moved_off_loopback());

    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].kind.as_str(), "port.listen.exposed");
    assert_eq!(fired[0].severity, Severity::High);
    assert_eq!(fired[0].finding_key, "port.listen|tcp|0.0.0.0:5432");
    assert!(
        fired[0].title.contains("from 127.0.0.1 to 0.0.0.0"),
        "{}",
        fired[0].title
    );
    assert!(fired[0].before.is_some() && fired[0].after.is_some());
}

#[test]
fn both_halves_of_the_move_are_claimed_so_nothing_reports_them_again() {
    let claimed = claimed_by(&moved_off_loopback());

    assert_eq!(
        claimed,
        BTreeSet::from(["tcp|0.0.0.0:5432".to_string(), "tcp|127.0.0.1:5432".into()]),
        "an unclaimed half becomes 'a new port appeared' next to 'a port went away'"
    );
}

#[test]
fn a_move_onto_the_v6_wildcard_is_the_same_move_even_though_it_is_another_table() {
    let mut v6 = fixture::socket("::", 8080, "/usr/sbin/nginx", "root");
    v6["protocol"] = serde_json::json!("tcp6");
    let changes = vec![
        Change::Added {
            key: "tcp6|:::8080".into(),
            after: v6,
        },
        Change::Added {
            key: "tcp|0.0.0.0:8080".into(),
            after: fixture::socket("0.0.0.0", 8080, "/usr/sbin/nginx", "root"),
        },
        Change::Removed {
            key: "tcp|127.0.0.1:8080".into(),
            before: fixture::socket("127.0.0.1", 8080, "/usr/sbin/nginx", "root"),
        },
    ];

    let fired = apply(&changes);

    assert_eq!(fired.len(), 1, "one move, whatever it left behind");
    assert_eq!(
        fired[0].finding_key, "port.listen|tcp|0.0.0.0:8080",
        "named after the row a person recognises, and the same on every reading"
    );
    assert!(
        fired[0]
            .evidence
            .iter()
            .any(|e| e.value.contains("0.0.0.0, ::")),
        "the other row has to be named or somebody goes looking for it: {:?}",
        fired[0].evidence
    );
    assert_eq!(claimed_by(&changes).len(), 3);
}

#[test]
fn a_binary_that_has_no_business_holding_a_port_makes_the_move_critical() {
    let changes = vec![
        Change::Added {
            key: "tcp|0.0.0.0:9001".into(),
            after: fixture::socket("0.0.0.0", 9001, "/dev/shm/payload", "www-data"),
        },
        Change::Removed {
            key: "tcp|127.0.0.1:9001".into(),
            before: fixture::socket("127.0.0.1", 9001, "/dev/shm/payload", "www-data"),
        },
    ];

    assert_eq!(apply(&changes)[0].severity, Severity::Critical);
}

#[test]
fn a_service_that_keeps_its_loopback_socket_has_not_moved() {
    let changes = vec![Change::Added {
        key: "tcp|0.0.0.0:8080".into(),
        after: fixture::socket("0.0.0.0", 8080, "/usr/sbin/nginx", "root"),
    }];

    assert!(apply(&changes).is_empty());
    assert!(claimed_by(&changes).is_empty());
}

#[test]
fn a_service_that_retreats_to_loopback_is_not_an_exposure() {
    let changes = vec![
        Change::Added {
            key: "tcp|127.0.0.1:8080".into(),
            after: fixture::socket("127.0.0.1", 8080, "/usr/sbin/nginx", "root"),
        },
        Change::Removed {
            key: "tcp|0.0.0.0:8080".into(),
            before: fixture::socket("0.0.0.0", 8080, "/usr/sbin/nginx", "root"),
        },
    ];

    assert!(apply(&changes).is_empty(), "that is somebody hardening it");
    assert!(claimed_by(&changes).is_empty());
}

#[test]
fn two_unrelated_ports_moving_at_once_are_two_findings_and_not_a_cross_product() {
    let mut changes = moved_off_loopback();
    changes.extend([
        Change::Added {
            key: "tcp|0.0.0.0:6379".into(),
            after: fixture::socket("0.0.0.0", 6379, "/usr/bin/redis", "redis"),
        },
        Change::Removed {
            key: "tcp|127.0.0.1:6379".into(),
            before: fixture::socket("127.0.0.1", 6379, "/usr/bin/redis", "redis"),
        },
    ]);

    let fired = apply(&changes);

    assert_eq!(fired.len(), 2);
    let keys: Vec<&str> = fired.iter().map(|f| f.finding_key.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "port.listen|tcp|0.0.0.0:5432",
            "port.listen|tcp|0.0.0.0:6379"
        ],
        "grouped by port, in a fixed order"
    );
}

#[test]
fn a_udp_socket_does_not_pair_with_a_tcp_one_on_the_same_number() {
    let changes = vec![
        Change::Added {
            key: "udp|0.0.0.0:53".into(),
            after: {
                let mut value = fixture::socket("0.0.0.0", 53, "/usr/sbin/dnsmasq", "dnsmasq");
                value["protocol"] = serde_json::json!("udp");
                value
            },
        },
        Change::Removed {
            key: "tcp|127.0.0.1:53".into(),
            before: fixture::socket("127.0.0.1", 53, "/usr/sbin/dnsmasq", "dnsmasq"),
        },
    ];

    assert!(apply(&changes).is_empty(), "two stacks, two services");
}

#[test]
fn a_unix_socket_never_takes_part_in_this_at_all() {
    let changes = vec![
        Change::Added {
            key: "unix|/run/x.sock".into(),
            after: fixture::unix_socket("/run/x.sock", "/usr/bin/x", "root"),
        },
        Change::Removed {
            key: "unix|/run/y.sock".into(),
            before: fixture::unix_socket("/run/y.sock", "/usr/bin/x", "root"),
        },
    ];

    assert!(
        apply(&changes).is_empty(),
        "it has no address to be exposed on"
    );
}
