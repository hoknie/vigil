use super::harness::drawn;
use crate::ui::{Reading, Startup, fixture};

#[test]
fn every_list_says_what_it_is_and_what_it_says_when_it_holds_nothing() {
    let mut view = fixture::view();
    view.readings.put(
        "persistence",
        Reading::Taken(
            vigil_model::Snapshot::new("persistence", "2026-09-09T09:00:00.000Z").with(
                "unit|nginx.service",
                serde_json::json!({"name": "nginx.service", "type": "service", "readable": true}),
            ),
        ),
    );

    let timers = drawn(&view, Startup::Timers, 80);

    assert!(timers.contains("No timer in this reading"), "{timers}");
    assert!(timers.contains("No systemd timer is set"), "{timers}");
}

#[test]
fn a_refusal_the_collector_stated_about_the_whole_reading_is_not_pinned_on_one_list() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut()
        && let Some(persistence) = status
            .agent
            .collectors
            .iter_mut()
            .find(|it| it.name == "persistence")
    {
        persistence.state = vigil_model::CollectorState::Degraded;
        persistence.reason = Some("/etc/cron.d cannot be read".into());
    }
    view.readings.put(
        "persistence",
        Reading::Taken(vigil_model::Snapshot::new(
            "persistence",
            "2026-09-09T09:00:00.000Z",
        )),
    );

    let page = drawn(&view, Startup::Timers, 80);

    assert!(page.contains("/etc/cron.d cannot be read"), "{page}");
    assert!(
        page.contains("holds nothing at all"),
        "an empty whole reading is a failed reading, not five empty lists: {page}"
    );
}

#[test]
fn the_row_of_lists_names_every_list_this_reading_produced() {
    let page = drawn(&fixture::view(), Startup::Cron, 80);

    assert!(page.contains("[cron]"), "{page}");
    assert!(page.contains("units"), "{page}");
    assert!(page.contains("modules"), "{page}");
    assert!(page.contains("files"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for list in Startup::ALL {
            let page = drawn(&fixture::view(), *list, width);
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}
