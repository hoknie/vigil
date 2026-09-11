use serde_json::json;
use vigil_model::Snapshot;

use super::harness::{drawn, drawn_as, showing};
use crate::ui::screens::startup::keys;
use crate::ui::{Nesting, Reading, Search, Startup, View, fixture};

fn nested() -> Nesting {
    let mut nesting = Nesting::default();
    nesting.toggle();
    nesting
}

fn chain(deep: usize) -> View {
    let mut snapshot = Snapshot::new("persistence", "2026-09-09T09:00:00.000Z");
    for step in 0..deep {
        snapshot = snapshot.with(
            format!("unit|step-{step:02}-a-rather-long-unit-name.service"),
            json!({
                "name": format!("step-{step:02}-a-rather-long-unit-name.service"),
                "type": "service", "readable": true, "commands": [],
                "wants": [format!("step-{:02}-a-rather-long-unit-name.service", step + 1)],
            }),
        );
    }
    let mut view = fixture::view();
    view.readings.put("persistence", Reading::Taken(snapshot));
    view
}

#[test]
fn the_tree_and_the_list_hold_the_same_units_and_only_their_order_differs() {
    let view = fixture::view();
    let search = Search::default();

    let mut flat = keys(&view, &showing(Startup::Units, &search, Nesting::default()));
    let mut tree = keys(&view, &showing(Startup::Units, &search, nested()));

    assert_ne!(
        flat, tree,
        "the whole point is that the order is not the same"
    );
    flat.sort();
    tree.sort();
    assert_eq!(
        flat, tree,
        "a view that holds fewer objects than its neighbour hides them"
    );
}

#[test]
fn a_unit_a_target_pulls_in_is_drawn_under_it_and_indented() {
    let page = drawn_as(
        &fixture::view(),
        showing(Startup::Units, &Search::default(), nested()),
        80,
    );

    let rows: Vec<&str> = page
        .lines()
        .filter(|line| line.contains(".target") || line.contains(".service"))
        .collect();
    let target = rows
        .iter()
        .position(|line| line.contains("multi-user.target"))
        .expect("the target is a row");
    let nginx = rows
        .iter()
        .position(|line| line.contains("nginx.service"))
        .expect("the service is a row");

    assert_eq!(nginx, target + 1, "{page}");
    assert!(
        rows[nginx].find("nginx").expect("it is there")
            > rows[target].find("multi-user").expect("it is there"),
        "the child is not indented under its parent: {page}"
    );
}

#[test]
fn a_unit_nothing_pulls_in_is_still_a_row_of_the_tree() {
    let page = drawn_as(
        &fixture::view(),
        showing(Startup::Units, &Search::default(), nested()),
        80,
    );

    assert!(page.contains("rescue-shell.service"), "{page}");
}

#[test]
fn the_key_that_switches_the_view_is_drawn_beside_the_list_it_switches_and_nowhere_else() {
    let view = fixture::view();
    let search = Search::default();

    let units = drawn_as(
        &view,
        showing(Startup::Units, &search, Nesting::default()),
        80,
    );
    assert!(units.contains("[t list]"), "{units}");

    for list in [
        Startup::Timers,
        Startup::Cron,
        Startup::Modules,
        Startup::Files,
    ] {
        let page = drawn(&view, list, 80);
        assert!(
            !page.contains("t tree"),
            "{} offers a key that does nothing there: {page}",
            list.name()
        );
    }
}

#[test]
fn the_tree_says_which_of_the_two_answers_it_is_showing() {
    let page = drawn_as(
        &fixture::view(),
        showing(Startup::Units, &Search::default(), nested()),
        80,
    );

    assert!(page.contains("as written in the files"), "{page}");
    assert!(
        page.contains("not what systemd has enabled"),
        "the tree has to name which of the two answers it draws: {page}"
    );
}

#[test]
fn a_unit_more_than_one_thing_pulls_in_is_marked_and_the_footer_says_what_the_mark_means() {
    let page = drawn_as(
        &fixture::view(),
        showing(Startup::Units, &Search::default(), nested()),
        80,
    );

    assert!(page.contains("nginx.service +1"), "{page}");
    assert!(
        page.contains("+N means N more pull it in"),
        "a mark with no key to it is a mark nobody can read: {page}"
    );
    assert_eq!(
        page.matches("nginx.service").count(),
        1,
        "a unit with two parents was drawn twice: {page}"
    );
}

#[test]
fn a_tree_deeper_than_the_indent_can_show_keeps_the_name_rather_than_the_depth() {
    let view = chain(12);

    for width in [80u16, 140] {
        let page = drawn_as(
            &view,
            showing(Startup::Units, &Search::default(), nested()),
            width,
        );
        let deepest = page
            .lines()
            .find(|line| line.contains("step-11"))
            .unwrap_or_else(|| panic!("{width} columns lost the deepest unit: {page}"));

        assert!(
            deepest.contains("step-11") && !deepest.contains("step-10"),
            "{width}: {deepest}"
        );
        assert!(
            deepest.contains('…') || deepest.contains("unit-name.service"),
            "a name that was cut has to say it was cut: {deepest}"
        );
    }
}

#[test]
fn nothing_runs_off_the_side_of_the_tree_at_any_width_this_is_read_at() {
    for width in [80u16, 120, 140, 200] {
        for view in [fixture::view(), chain(12)] {
            let page = drawn_as(
                &view,
                showing(Startup::Units, &Search::default(), nested()),
                width,
            );
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}
