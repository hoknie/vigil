use super::harness::drawn;
use crate::link::{Trouble, TroubleKind};
use crate::ui::{Screen, View, fixture};

#[test]
fn without_an_answer_the_title_says_where_it_was_looking() {
    let view = View::nothing_yet("/run/vigil/vigil.sock");

    let (page, _) = drawn(fixture::look(), Screen::SUMMARY, &view, 80, 24);

    assert!(
        page.contains("no answer from /run/vigil/vigil.sock"),
        "{page}"
    );
    assert!(page.contains("waiting for the agent"), "{page}");
}

#[test]
fn a_console_that_asked_and_was_not_answered_does_not_say_it_is_still_waiting() {
    let mut view = View::nothing_yet("/run/vigil/vigil.sock");
    view.trouble = Some(Trouble::new(
        "/run/vigil/vigil.sock",
        TroubleKind::Absent,
        "No such file or directory (os error 2)",
    ));

    let (page, _) = drawn(fixture::look(), Screen::SUMMARY, &view, 80, 24);

    assert!(page.contains("is not answering"), "{page}");
    assert!(!page.contains("waiting for the agent"), "{page}");
}
