use super::harness::drawn;
use crate::link::{Trouble, TroubleKind};
use crate::ui::{Screen, fixture};

#[test]
fn a_reading_that_is_no_longer_live_says_so_in_the_frame_of_every_screen() {
    let mut view = fixture::view();
    view.trouble = Some(Trouble::new(
        "/run/vigil/vigil.sock",
        TroubleKind::Absent,
        "No such file or directory (os error 2)",
    ));

    for screen in Screen::all() {
        let (page, _) = drawn(fixture::look(), screen, &view, 80, 24);
        assert!(
            page.contains("NOT ANSWERING"),
            "{} said nothing about it: {page}",
            screen.name()
        );
        assert!(
            page.contains("09:00:01"),
            "and did not say how old this is: {page}"
        );
    }
}

#[test]
fn a_live_screen_says_when_the_reading_on_it_was_taken() {
    let (page, _) = drawn(fixture::look(), Screen::SUMMARY, &fixture::view(), 80, 24);

    assert!(page.contains("as of 09:00:01"), "{page}");
    assert!(!page.contains("NOT ANSWERING"), "{page}");
}
