use serde_json::json;
use vigil_model::Snapshot;

use super::harness::drawn;
use crate::ui::{Reading, Startup, View, fixture};

fn holding(key: &str, item: serde_json::Value) -> View {
    let mut view = fixture::view();
    view.readings.put(
        "persistence",
        Reading::Taken(Snapshot::new("persistence", "2026-09-09T09:00:00.000Z").with(key, item)),
    );
    view
}

fn cells(page: &str, name: &str) -> String {
    page.lines()
        .find(|line| line.contains(name))
        .unwrap_or_else(|| panic!("no row for {name}: {page}"))
        .to_string()
}

#[test]
fn a_module_size_is_read_in_the_units_a_person_uses() {
    let page = drawn(&fixture::view(), Startup::Modules, 80);

    assert!(cells(&page, "overlay").contains("152.0 KB"), "{page}");
    assert!(
        !page.contains("155648"),
        "a size in bytes is a number nobody reads at a glance: {page}"
    );
}

#[test]
fn a_module_that_is_not_empty_is_never_rounded_down_to_nothing() {
    let view = holding(
        "module|tiny",
        json!({"name": "tiny", "size": 512, "dependencies": [], "state": "Live"}),
    );

    let page = drawn(&view, Startup::Modules, 80);

    assert!(cells(&page, "tiny").contains("512 B"), "{page}");
    assert!(
        !page.contains("0.0 KB"),
        "a module that holds something drawn as nothing is the same lie as an empty list: \
         {page}"
    );
}

#[test]
fn the_column_of_sizes_lines_up_on_its_right_edge() {
    let mut view = fixture::view();
    view.readings.put(
        "persistence",
        Reading::Taken(
            Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
                .with(
                    "module|tiny",
                    json!({"name": "tiny", "size": 512, "dependencies": [], "state": "Live"}),
                )
                .with(
                    "module|large",
                    json!({"name": "large", "size": 155_648, "dependencies": [], "state": "Live"}),
                ),
        ),
    );

    let page = drawn(&view, Startup::Modules, 80);
    let small = cells(&page, "tiny");
    let big = cells(&page, "large");

    assert_eq!(
        small.find(" B").map(|at| at + 2),
        big.find(" KB").map(|at| at + 3),
        "a column of numbers that does not line up stops being a column:\n{page}"
    );
}

#[test]
fn a_timer_with_more_than_one_schedule_says_there_are_more_of_them() {
    let page = drawn(&fixture::view(), Startup::Timers, 80);

    let row = cells(&page, "logrotate.timer");
    assert!(row.contains("daily and 1 more"), "{page}");
    assert!(
        !row.contains("daily  "),
        "one of several schedules drawn alone reads as the only one: {page}"
    );
}

#[test]
fn the_when_column_holds_a_whole_schedule_at_eighty_columns() {
    let view = holding(
        "timer|backup.timer",
        json!({
            "name": "backup.timer", "type": "timer", "readable": true,
            "path": "/etc/systemd/system/backup.timer", "description": "Nightly backup",
            "on_calendar": ["Mon,Tue,Wed,Thu,Fri *-*-* 03:15:00"], "on_boot": null,
            "activates": "backup.service",
        }),
    );

    let page = drawn(&view, Startup::Timers, 80);

    assert!(
        cells(&page, "backup.timer").contains("Mon,Tue,Wed,Thu,Fri *-*-* 03:15:00"),
        "the narrowest terminal is the one this is read on during an incident: {page}"
    );
}

#[test]
fn a_timer_that_fires_from_boot_says_so_rather_than_saying_nothing_is_stated() {
    let view = holding(
        "timer|delayed.timer",
        json!({
            "name": "delayed.timer", "type": "timer", "readable": true,
            "path": "/etc/systemd/system/delayed.timer", "description": "After boot",
            "on_calendar": [], "on_boot": "15min", "activates": "delayed.service",
        }),
    );

    let page = drawn(&view, Startup::Timers, 80);

    assert!(cells(&page, "delayed.timer").contains("on boot"), "{page}");
    assert!(
        !page.contains("not stated in the file"),
        "the agent recorded a delay and the screen said it recorded nothing: {page}"
    );
}
