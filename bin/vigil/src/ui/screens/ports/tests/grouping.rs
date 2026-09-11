use serde_json::json;
use vigil_model::Snapshot;

use super::harness::{busy, drawn_with, showing};
use crate::ui::screens::ports::{Arrangement, What, rows};
use crate::ui::{Protocols, Reading, fixture};

#[test]
fn two_programs_with_the_same_file_name_are_never_shown_as_the_same_heading() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(
            Snapshot::new("ports", "2026-09-09T09:00:00.000Z")
                .with(
                    "tcp|0.0.0.0:8080",
                    json!({
                        "protocol": "tcp", "address": "0.0.0.0", "port": 8080, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/usr/bin/nc", "exe_deleted": false},
                        "owner_resolved": true,
                    }),
                )
                .with(
                    "tcp|0.0.0.0:5555",
                    json!({
                        "protocol": "tcp", "address": "0.0.0.0", "port": 5555, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/tmp/.x/nc", "exe_deleted": false},
                        "owner_resolved": true,
                    }),
                ),
        ),
    );

    let page = drawn_with(
        &view,
        &showing(Protocols::default(), Arrangement::ByProgram),
        0,
        100,
    );

    assert!(page.contains("/tmp/.x/nc (1)"), "{page}");
    assert!(page.contains("/usr/bin/nc (1)"), "{page}");
}

#[test]
fn grouped_by_program_puts_a_programs_sockets_under_one_heading() {
    let page = drawn_with(
        &busy(),
        &showing(Protocols::default(), Arrangement::ByProgram),
        0,
        120,
    );

    assert!(page.contains("nginx (2)"), "{page}");
    assert!(page.contains("dnsmasq (1)"), "{page}");
    assert!(
        page.contains("/usr/sbin/nginx"),
        "the path fits here: {page}"
    );
    assert!(page.contains("grouped by program"), "{page}");
}

#[test]
fn a_socket_whose_holder_could_not_be_found_is_never_grouped_with_the_programs() {
    let host = busy();
    let grouped = showing(Protocols::default(), Arrangement::ByProgram);
    let rows = rows(&host, &grouped.showing(0));

    let headings: Vec<&str> = rows
        .iter()
        .filter_map(|row| match &row.what {
            What::Program { path, .. } => Some(path.as_str()),
            What::Unresolved { .. } => Some("unresolved"),
            What::Socket(_) => None,
        })
        .collect();

    assert_eq!(
        headings,
        vec!["/usr/sbin/dnsmasq", "/usr/sbin/nginx", "unresolved"]
    );

    let page = drawn_with(
        &busy(),
        &showing(Protocols::default(), Arrangement::ByProgram),
        0,
        120,
    );
    assert!(page.contains("owner not resolved (1)"), "{page}");
    assert!(page.contains("permission denied"), "{page}");
}
