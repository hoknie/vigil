use super::harness::drawn;
use crate::link::{Trouble, TroubleKind};
use crate::ui::{Screen, fixture};

#[test]
fn a_collector_that_is_off_is_counted_apart_from_one_that_is_not_reading() {
    let view = fixture::view();

    let (page, _) = drawn(fixture::look(), Screen::SUMMARY, &view, 120, 24);

    assert!(page.contains("10 collector(s), 9 reading, 1 off"), "{page}");
    assert!(!page.contains("not reading everything"), "{page}");
}

#[test]
fn a_collector_that_is_off_and_one_that_is_failing_are_two_different_numbers() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].state = vigil_model::CollectorState::Degraded;
    }

    let (page, _) = drawn(fixture::look(), Screen::SUMMARY, &view, 120, 24);

    assert!(
        page.contains("10 collector(s), 1 not reading everything, 1 off"),
        "{page}"
    );
}

#[test]
fn a_status_bar_with_no_room_drops_a_whole_fact_rather_than_half_a_word() {
    let mut view = fixture::view();
    view.trouble = Some(Trouble::new(
        "/run/vigil/vigil.sock",
        TroubleKind::Absent,
        "No such file or directory (os error 2)",
    ));

    let (narrow, _) = drawn(fixture::look(), Screen::SUMMARY, &view, 60, 24);

    assert!(
        narrow.contains("NOT ANSWERING"),
        "the part that must survive did not: {narrow}"
    );
    assert!(
        !narrow.lines().any(|line| line.ends_with('·')),
        "a separator with nothing after it is a sentence that was cut: {narrow}"
    );
}
