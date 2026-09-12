use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;
use vigil_model::{CollectorRefusal, CollectorState, Snapshot};

use super::{Showing, render};
use crate::ui::helpers::words::text;
use crate::ui::{Arrows, Reading, Refusal, Search, Sorting, View, fixture};
use vigil_model::Severity;

fn showing(search: &Search) -> Showing<'_> {
    Showing {
        search,
        cursor: 0,
        arrows: Arrows::List,
        gone: None,
        sorting: Sorting::default(),
    }
}

fn drawn(view: &View, width: u16) -> String {
    let search = Search::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 24));
    render(
        view,
        fixture::look(),
        &showing(&search),
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn a_container_says_what_it_runs_what_it_may_do_and_how_much_of_the_host_it_holds() {
    let page = drawn(&fixture::view(), 120);

    assert!(page.contains("3ab1c0f2d4e5"), "{page}");
    assert!(page.contains("/usr/sbin/nginx"), "{page}");
    assert!(page.contains("docker"), "{page}");
    assert!(page.contains("/run/docker.sock"), "the socket too: {page}");
}

#[test]
fn the_container_that_may_leave_the_container_is_told_apart_from_the_one_that_may_not() {
    let page = drawn(&fixture::view(), 80);
    let ordinary = page
        .lines()
        .find(|line| line.contains("3ab1c0f2d4e5"))
        .expect("the ordinary container");
    let privileged = page
        .lines()
        .find(|line| line.contains("9f2e8d7c6b5a"))
        .expect("the privileged container");

    assert!(ordinary.contains("no"), "{ordinary}");
    assert!(privileged.contains("yes"), "{privileged}");
    assert!(
        page.contains("SYS_ADMIN"),
        "and the column says which capability that is: {page}"
    );
}

#[test]
fn a_capability_set_that_could_not_be_read_is_never_drawn_as_an_ordinary_container() {
    let mut view = fixture::view();
    view.readings.put(
        "containers",
        Reading::Taken(
            Snapshot::new("containers", "2026-09-09T09:00:00.000Z").with(
                "container|3ab1c0f2d4e5",
                json!({
                    "id": "3ab1c0f2d4e5", "runtime": "docker", "exe": "/usr/sbin/nginx",
                    "capabilities_effective": null, "host_paths": [],
                    "host_paths_truncated": false, "mounts_readable": true,
                }),
            ),
        ),
    );

    let page = drawn(&view, 80);
    let row = page
        .lines()
        .find(|line| line.contains("3ab1c0f2d4e5"))
        .expect("the container");

    assert!(row.contains('?'), "{row}");
    assert!(!row.contains(" no "), "{row}");
}

#[test]
fn a_host_running_no_container_says_that_rather_than_drawing_an_empty_table() {
    let mut view = fixture::view();
    view.readings.put(
        "containers",
        Reading::Taken(Snapshot::new("containers", "2026-09-09T09:00:00.000Z")),
    );

    let page = drawn(&view, 80);

    assert!(page.contains("Nothing is running in a container"), "{page}");
    assert!(page.contains("not a reading that failed"), "{page}");
}

#[test]
fn a_reading_that_was_refused_is_a_screen_that_says_so_and_not_an_empty_table() {
    let mut view = fixture::view();
    view.readings.put(
        "containers",
        Reading::Refused(Refusal::told(CollectorRefusal::new(
            CollectorState::Degraded,
            "the runtime socket is there and this agent may not read it",
        ))),
    );

    let page = drawn(&view, 80);

    assert!(page.contains("refused"), "{page}");
    assert!(page.contains("may not read it"), "{page}");
}

#[test]
fn the_table_starts_at_the_top_and_carries_no_banner_over_it() {
    let page = drawn(&fixture::view(), 80);
    let heading = page
        .lines()
        .position(|line| line.contains("CONTAINER") && line.contains("SYS_ADMIN"))
        .expect("a heading");

    assert!(heading <= 2, "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for line in drawn(&fixture::view(), width).lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}

#[test]
fn the_section_is_about_containers_and_the_runtime_socket_is_named_under_the_table() {
    let page = drawn(&fixture::view(), 80);

    assert!(
        !page
            .lines()
            .any(|line| line.contains("socket") && line.contains(" > ")),
        "a socket is not a container, and a row for it made every container carry a dash in \
         a column drawn for it: {page}"
    );
    assert!(
        page.contains("/run/docker.sock"),
        "the socket is still read on this screen, because a finding about it is about this \
         host: {page}"
    );
    assert!(
        page.contains("/run/docker.sock 0660"),
        "with the mode that decides who may write to it: {page}"
    );
}

#[test]
fn a_host_with_a_runtime_and_nothing_running_says_which_of_the_two_it_is() {
    let mut view = fixture::view();
    view.readings.put(
        "containers",
        Reading::Taken(
            Snapshot::new("containers", "2026-09-09T09:00:00.000Z").with(
                "container-socket|/run/docker.sock",
                json!({"path": "/run/docker.sock", "mode": "0660", "uid": 0, "gid": 999}),
            ),
        ),
    );

    let page = drawn(&view, 120);

    assert!(page.contains("Nothing is running in a container"), "{page}");
}

#[test]
fn no_column_of_this_table_holds_a_dash_on_every_row() {
    let page = drawn(&fixture::view(), 120);
    let heading = page
        .lines()
        .find(|line| line.contains("CONTAINER"))
        .expect("a heading");
    let rows: Vec<&str> = page
        .lines()
        .filter(|line| line.contains("3ab1c0f2d4e5") || line.contains("9f2e8d7c6b5a"))
        .collect();

    for (at, _) in heading.char_indices().filter(|(_, letter)| *letter != ' ') {
        let empty = rows
            .iter()
            .all(|row| row.chars().nth(at).is_some_and(|letter| letter == '—'));
        assert!(!empty, "a column of dashes is worse than no column: {page}");
    }
}

#[test]
fn a_finding_about_the_runtime_socket_is_about_the_container_that_holds_it() {
    use crate::ui::Anchor;

    let mut finding = fixture::finding("a container holds the runtime socket", Severity::Critical);
    finding.finding_key = "container|docker-socket|/usr/local/bin/agent".into();

    let anchor = Anchor::of(&finding).expect("a container to walk to");

    assert_eq!(anchor.screen, crate::ui::Screen::Containers);
    assert_eq!(
        anchor.key, "/usr/local/bin/agent",
        "the rule keys that finding by the container, not by the socket, so taking the \
         socket rows off the table takes nothing away from the jump"
    );
}
