use serde_json::json;
use vigil_model::Snapshot;

use super::harness::{Given, busy, drawn};
use crate::ui::screens::ports::keys;
use crate::ui::{Reading, fixture};

#[test]
fn a_socket_shows_its_endpoint_its_user_and_the_program_holding_it() {
    let page = drawn(&fixture::view(), 0, 120);

    assert!(page.contains("0.0.0.0:4444"), "{page}");
    assert!(page.contains("www-data"), "{page}");
    assert!(
        page.contains("nc"),
        "the program by name, not by path: {page}"
    );
    assert!(page.contains("(deleted)"), "{page}");
}

#[test]
fn the_column_carries_the_programs_name_and_the_detail_carries_its_path() {
    let page = drawn(&busy(), 0, 80);

    assert!(page.contains("nginx"), "{page}");
    assert!(
        !page.contains("/usr/sbin/nginx"),
        "the path belongs on the detail: {page}"
    );
}

#[test]
fn a_wide_terminal_buys_the_command_line_and_a_narrow_one_does_not_pretend_to() {
    let wide = drawn(&fixture::view(), 0, 200);
    let narrow = drawn(&fixture::view(), 0, 80);

    assert!(wide.contains("nc -l -p 4444"), "{wide}");
    assert!(wide.contains("COMMAND"), "{wide}");
    assert!(
        !narrow.contains("COMMAND"),
        "a column with no room for a value in it is worse than no column: {narrow}"
    );
}

#[test]
fn a_value_the_column_could_not_hold_says_that_it_was_cut() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(Snapshot::new("ports", "2026-09-09T09:00:00.000Z").with(
            "unix|/run/a/very/long/path/that/will/not/fit/in/the/address/column.sock",
            json!({
                "protocol": "unix", "type": "stream",
                "path": "/run/a/very/long/path/that/will/not/fit/in/the/address/column.sock",
                "abstract": false, "uid": 0, "user": "root",
                "process": {"exe": "/usr/bin/dockerd", "exe_deleted": false},
                "owner_resolved": true,
            }),
        )),
    );

    assert!(
        drawn(&view, 0, 80).lines().any(|line| line.contains('…')),
        "a row that quietly loses its tail is a row that lies"
    );
}

#[test]
fn the_keys_are_the_snapshots_own_so_a_jump_from_a_finding_lands_on_the_right_row() {
    assert_eq!(
        keys(&fixture::view(), &Given::default().showing(0)),
        vec![
            "tcp6|:::443".to_string(),
            "tcp6|:::8080".to_string(),
            "tcp|0.0.0.0:443".to_string(),
            "tcp|0.0.0.0:4444".to_string(),
            "tcp|0.0.0.0:9000".to_string(),
            "tcp|127.0.0.1:5432".to_string(),
            "udp6|:::5353".to_string(),
            "udp6|:::546".to_string(),
            "udp|0.0.0.0:53".to_string(),
            "udp|0.0.0.0:68".to_string(),
            "unix|/run/docker.sock".to_string(),
            "unix|@/tmp/.X11-unix/X0".to_string(),
            "unix|unnamed".to_string(),
        ]
    );
}
