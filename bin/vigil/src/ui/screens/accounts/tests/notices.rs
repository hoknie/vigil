use vigil_model::Snapshot;

use super::harness::{degrade, drawn, drawn_at, strip_sessions};
use crate::ui::{Reading, Subject, fixture};

#[test]
fn an_empty_list_under_a_collector_that_is_not_reading_everything_is_not_an_empty_host() {
    let mut view = fixture::view();
    strip_sessions(&mut view);
    degrade(
        &mut view,
        "logins are not recorded in a form this build reads — active sessions will be missing",
    );

    let page = drawn_at(&view, Subject::LoggedIn, 120);

    assert!(page.contains("reading is incomplete"), "{page}");
    assert!(page.contains("logins are not recorded"), "{page}");
    assert!(!page.contains("Nobody is logged in"), "{page}");
}

#[test]
fn an_empty_list_with_a_collector_that_is_fine_says_it_read_and_found_nothing() {
    let mut view = fixture::view();
    strip_sessions(&mut view);

    let page = drawn_at(&view, Subject::LoggedIn, 120);

    assert!(page.contains("No session in this reading"), "{page}");
    assert!(page.contains("login records"), "{page}");
}

#[test]
fn a_reading_that_has_not_happened_is_not_a_host_nobody_can_log_in_to() {
    let mut view = fixture::view();
    view.readings.put("users", Reading::NotTakenYet);

    let page = drawn(&view, Subject::Users);

    assert!(page.contains("has not read the accounts yet"), "{page}");
    assert!(
        !page.contains("PASSWORD"),
        "an empty table would be a lie: {page}"
    );
    assert!(
        page.contains("users"),
        "and the submenu is still there to move by: {page}"
    );
}

#[test]
fn a_reading_with_no_account_in_it_is_called_a_reading_to_distrust() {
    let mut view = fixture::view();
    view.readings.put(
        "users",
        Reading::Taken(Snapshot::new("users", "2026-09-09T09:00:00.000Z")),
    );

    assert!(drawn(&view, Subject::Users).contains("Every host has accounts"));
}
