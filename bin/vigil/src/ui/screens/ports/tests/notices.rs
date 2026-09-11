use vigil_model::{CollectorRefusal, CollectorState, Snapshot};

use super::harness::drawn;
use crate::ui::{Reading, Refusal, fixture};

#[test]
fn a_reading_that_has_not_happened_is_not_drawn_as_a_host_with_nothing_on_it() {
    let mut view = fixture::view();
    view.readings.put("ports", Reading::NotTakenYet);

    let page = drawn(&view, 0, 80);

    assert!(page.contains("has not read the sockets yet"), "{page}");
    assert!(
        !page.contains("PROTO"),
        "an empty table would be a lie: {page}"
    );
}

#[test]
fn a_reading_nobody_has_asked_for_is_not_the_same_screen_as_one_that_came_back_empty() {
    let mut view = fixture::view();
    view.readings.put("ports", Reading::Unknown);
    assert!(drawn(&view, 0, 80).contains("has not been asked yet"));

    view.readings.put(
        "ports",
        Reading::Taken(Snapshot::new("ports", "2026-09-09T09:00:00.000Z")),
    );
    assert!(drawn(&view, 0, 80).contains("found no listening sockets"));
}

#[test]
fn a_refusal_is_loud_and_carries_the_agents_own_words() {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Refused(Refusal::answered(
            "this build reads status, snapshot, findings",
        )),
    );

    let page = drawn(&view, 0, 80);

    assert!(page.contains("reading was refused"), "{page}");
    assert!(page.contains("this build reads"), "{page}");
}

#[test]
fn a_collector_that_could_not_read_is_not_a_collector_whose_turn_has_not_come() {
    let mut waiting = fixture::view();
    waiting.readings.put("ports", Reading::NotTakenYet);
    let mut unable = fixture::view();
    unable.readings.put(
        "ports",
        Reading::Refused(Refusal::told(CollectorRefusal::new(
            CollectorState::Unavailable,
            "/proc/net/tcp cannot be read (Permission denied)",
        ))),
    );

    let soon = drawn(&waiting, 0, 80);
    let never = drawn(&unable, 0, 80);

    assert!(soon.contains("has not read the sockets yet"), "{soon}");
    assert!(
        !never.contains("has not read the sockets yet"),
        "a refusal must not be drawn as a reading that is on its way: {never}"
    );
    assert!(never.contains("ports is unavailable"), "{never}");
    assert!(never.contains("Permission denied"), "{never}");
    assert!(
        never.contains("No listening socket is listed here"),
        "and it says what is missing because of it: {never}"
    );
}
