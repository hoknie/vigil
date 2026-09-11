use super::harness::{drawn_with, holding, opened_on, squashed};
use crate::ui::fixture;

#[test]
fn a_finding_about_a_row_that_is_gone_opens_a_screen_that_says_so() {
    let flushed = opened_on(
        "fw-table|inet filter",
        "The inet table filter was deleted with 14 rule(s) in it",
    );
    let view = holding(fixture::firewall::filtering_nothing());

    let page = drawn_with(&view, &flushed, 0, 80, fixture::look());

    assert!(
        squashed(&page).contains(&squashed(
            "fw-table|inet filter is not in this reading any more"
        )),
        "{page}"
    );
    assert!(
        squashed(&page).contains(&squashed(
            "The inet table filter was deleted with 14 rule(s)"
        )),
        "the words of the finding are what a reader came here for: {page}"
    );
    assert!(
        squashed(&page).contains(&squashed("last had that row in front of it at")),
        "and when the agent last saw the row, because the reading cannot say: {page}"
    );
    assert!(
        squashed(&page).contains(&squashed(
            "that is the event itself, not a failure to find it"
        )),
        "a flushed table is gone by the time the finding is opened, and a screen that treats \
         that as a miss teaches a reader to distrust the trail: {page}"
    );
    assert!(
        page.contains("KIND")
            || squashed(&page).contains(&squashed("No chain is on the input hook")),
        "and what the reading holds now is still under it: {page}"
    );
}

#[test]
fn the_screen_with_no_row_missing_says_nothing_about_one() {
    let page = drawn_with(
        &fixture::view(),
        &super::harness::Given::default(),
        0,
        80,
        fixture::look(),
    );

    assert!(
        !squashed(&page).contains(&squashed("is not in this reading any more")),
        "{page}"
    );
}

#[test]
fn a_row_the_finding_is_about_and_that_is_still_there_is_opened_on_the_row_itself() {
    let page = drawn_with(
        &fixture::view(),
        &super::harness::Given::default(),
        3,
        80,
        fixture::look(),
    );

    assert!(
        page.lines().any(|line| line.starts_with(" > ")),
        "the cursor lands on a row, and the panel beside it is the row's own detail: {page}"
    );
}
