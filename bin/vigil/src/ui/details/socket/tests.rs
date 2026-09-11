use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;
use vigil_model::Snapshot;

use crate::ui::details::socket::render;
use crate::ui::helpers::words::text as page;
use crate::ui::screens::ports;
use crate::ui::screens::ports::Arrangement;
use crate::ui::{Reading, fixture};

fn drawn(view: &crate::ui::View, at: usize, width: u16) -> String {
    drawn_grouped(view, Arrangement::Flat, at, width)
}

fn drawn_keyed(view: &crate::ui::View, key: &str, width: u16) -> String {
    let protocols = crate::ui::Protocols::default();
    let search = crate::ui::Search::default();
    let listed = ports::rows(
        view,
        &ports::Showing {
            arrangement: Arrangement::Flat,
            protocols: &protocols,
            search: &search,
            cursor: 0,
            arrows: crate::ui::Arrows::Away,
        },
    );
    let at = listed
        .iter()
        .position(|row| row.key == key)
        .unwrap_or_else(|| panic!("no row keyed {key}"));

    drawn_grouped(view, Arrangement::Flat, at, width)
}

fn drawn_grouped(
    view: &crate::ui::View,
    arrangement: Arrangement,
    at: usize,
    width: u16,
) -> String {
    let protocols = crate::ui::Protocols::default();
    let search = crate::ui::Search::default();
    let rows = ports::rows(
        view,
        &ports::Showing {
            arrangement,
            protocols: &protocols,
            search: &search,
            cursor: 0,
            arrows: crate::ui::Arrows::Away,
        },
    );
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 40));
    render(rows.get(at), fixture::look(), 0, buffer.area, &mut buffer);
    page::to_text(&buffer)
}

#[test]
fn it_carries_the_whole_path_the_table_had_no_room_for() {
    let view = fixture::view();

    let detail = drawn_keyed(&view, "tcp|0.0.0.0:4444", 60);

    assert!(detail.contains("/tmp/.x/nc"), "{detail}");
    assert!(
        detail.contains("nc -l -p 4444"),
        "the command line too: {detail}"
    );
    assert!(detail.contains("www-data"), "{detail}");
    assert!(detail.contains("uid"), "{detail}");
    assert!(detail.contains("tcp"), "{detail}");
    assert!(detail.contains("4444"), "{detail}");
}

#[test]
fn a_binary_deleted_from_disk_is_called_the_finding_that_it_is() {
    let detail = drawn_keyed(&fixture::view(), "tcp|0.0.0.0:4444", 60);

    assert!(detail.contains("unlinked"), "{detail}");
}

#[test]
fn a_socket_whose_holder_was_out_of_reach_says_that_and_not_that_nothing_holds_it() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(Snapshot::new("ports", "2026-09-09T09:00:00.000Z").with(
            "tcp|0.0.0.0:22",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 22, "uid": 0,
                "user": null, "process": null, "owner_resolved": false,
            }),
        )),
    );

    let detail = drawn(&view, 0, 60);

    assert!(detail.contains("Not resolved"), "{detail}");
    let unbroken = detail.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        unbroken.contains("not the same as nothing holding it"),
        "{detail}"
    );
}

#[test]
fn a_command_line_with_something_scrubbed_out_of_it_says_so() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(Snapshot::new("ports", "2026-09-09T09:00:00.000Z").with(
            "tcp|0.0.0.0:3306",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 3306, "uid": 0, "user": "root",
                "process": {
                    "exe": "/usr/bin/mysql", "exe_deleted": false,
                    "cmdline": "mysql -u root -p", "cmdline_redacted": true,
                },
                "owner_resolved": true,
            }),
        )),
    );

    assert!(drawn(&view, 0, 60).contains("hidden on this host"));
}

#[test]
fn it_spells_the_configuration_entry_that_would_silence_this_socket() {
    let detail = drawn_keyed(&fixture::view(), "tcp|0.0.0.0:4444", 70);

    assert!(detail.contains("suppressions:"), "{detail}");
    assert!(
        detail.contains("finding_key: \"port.listen|tcp|0.0.0.0:4444\""),
        "{detail}"
    );
    assert!(detail.contains("reason:"), "{detail}");
}

#[test]
fn with_nothing_selected_it_says_how_to_select_something() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 10));
    render(None, fixture::look(), 0, buffer.area, &mut buffer);

    assert!(page::to_text(&buffer).contains("press →"));
}

#[test]
fn the_heading_of_a_group_the_agent_could_not_resolve_is_not_a_program_called_unknown() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(
            Snapshot::new("ports", "2026-09-09T09:00:00.000Z")
                .with(
                    "tcp|0.0.0.0:80",
                    json!({
                        "protocol": "tcp", "address": "0.0.0.0", "port": 80, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false},
                        "owner_resolved": true,
                    }),
                )
                .with(
                    "tcp|127.0.0.11:41857",
                    json!({
                        "protocol": "tcp", "address": "127.0.0.11", "port": 41857, "uid": 0,
                        "user": null, "process": null, "owner_resolved": false,
                    }),
                ),
        ),
    );

    let detail = drawn_grouped(&view, Arrangement::ByProgram, 2, 60);

    assert!(detail.contains("OWNER NOT RESOLVED"), "{detail}");
    assert!(detail.contains("not a program called"), "{detail}");

    let program = drawn_grouped(&view, Arrangement::ByProgram, 0, 60);
    assert!(program.contains("/usr/sbin/nginx"), "{program}");
    assert!(program.contains("sockets"), "{program}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [56u16, 80, 120] {
        for line in drawn_keyed(&fixture::view(), "tcp|0.0.0.0:4444", width).lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
