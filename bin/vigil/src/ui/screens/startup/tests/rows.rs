use super::harness::{drawn, showing};
use crate::ui::screens::startup::{keys, rows};
use crate::ui::{Nesting, Reading, Search, Startup, fixture};

#[test]
fn every_key_of_the_persistence_reading_is_a_row_of_exactly_one_list() {
    let view = fixture::view();
    let Reading::Taken(snapshot) = view.reading("persistence") else {
        panic!("the fixture holds a persistence reading");
    };

    let mut placed = 0usize;
    for list in Startup::on(&view) {
        placed += keys(
            &view,
            &showing(list, &Search::default(), Nesting::default()),
        )
        .len();
    }

    assert_eq!(
        placed,
        snapshot.items.len(),
        "an object with no list is an object no suppression can be written against"
    );
}

#[test]
fn a_cron_job_shows_the_whole_command_because_the_whole_command_is_the_key() {
    let page = drawn(&fixture::view(), Startup::Cron, 120);

    assert!(page.contains("/usr/local/bin/backup --to /srv"), "{page}");
    assert!(page.contains("/tmp/.x/implant"), "{page}");
    assert!(page.contains("*/5 * * * *"), "{page}");
}

#[test]
fn the_one_preload_file_is_a_row_and_not_a_list_of_its_own() {
    let view = fixture::view();

    let named = keys(
        &view,
        &showing(Startup::Files, &Search::default(), Nesting::default()),
    );

    assert!(
        named.contains(&"preload|/etc/ld.so.preload".to_string()),
        "{named:?}"
    );
    assert!(
        named.contains(&"script|/etc/profile".to_string()),
        "{named:?}"
    );
    assert!(
        !Startup::on(&view)
            .iter()
            .any(|list| list.name() == "preload"),
        "a list of one row teaches a reader not to press the row of lists"
    );
}

#[test]
fn a_row_that_says_the_modules_were_not_readable_is_sorted_to_the_top_and_not_counted() {
    let mut view = fixture::view();
    view.readings.put(
        "persistence",
        Reading::Taken(
            vigil_model::Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
                .with(
                    "module|overlay",
                    serde_json::json!({"name": "overlay", "size": 1, "dependencies": [], "state": "Live"}),
                )
                .with(
                    "modules|unreadable",
                    serde_json::json!({"readable": false, "reason": "/proc/modules could not be read"}),
                ),
        ),
    );

    let listed = rows(
        &view,
        &showing(Startup::Modules, &Search::default(), Nesting::default()),
    );

    assert!(listed[0].mark(), "the refusal comes first");
    let page = drawn(&view, Startup::Modules, 80);
    assert!(page.contains("/proc/modules could not be read"), "{page}");
}
