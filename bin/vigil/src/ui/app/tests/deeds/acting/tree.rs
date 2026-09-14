use ratatui::crossterm::event::KeyCode;

use super::sockets::{BY_PROGRAM, on_the_sockets};
use crate::ui::Level;
use crate::ui::app::tests::harness::{drawn_at, press};

#[test]
fn a_tree_arrives_folded_and_one_arrow_opens_one_branch_and_the_next_opens_the_row() {
    let mut app = on_the_sockets(BY_PROGRAM);

    let folded = drawn_at(&app, 120, 30);
    assert!(folded.contains("nginx"), "{folded}");
    assert!(!folded.contains(":::443"), "{folded}");

    press(&mut app, KeyCode::Right);
    let opened = drawn_at(&app, 120, 30);
    assert!(
        opened.contains(":::443") || opened.contains("0.0.0.0"),
        "the first arrow shows what is under the heading: {opened}"
    );
    assert_eq!(
        app.level,
        Level::List,
        "and it is still the list that has the arrows, not the detail panel"
    );

    press(&mut app, KeyCode::Right);
    assert!(
        drawn_at(&app, 120, 30).contains("THE SELECTED ROW"),
        "the second arrow shows the panel"
    );
    assert_eq!(
        app.level,
        Level::List,
        "and the arrows are still the list's: a reader who wanted the next row has not been \
         moved into a page of prose to get back out of"
    );

    press(&mut app, KeyCode::Right);
    assert_eq!(
        app.level,
        Level::Detail,
        "the third arrow is the one that hands the arrows to the panel"
    );
}

#[test]
fn the_left_arrow_undoes_that_in_the_same_order_it_was_done() {
    let mut app = on_the_sockets(BY_PROGRAM);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, KeyCode::Left);
    assert_eq!(
        app.level,
        Level::List,
        "the arrows come back to the list first, with the panel still up"
    );

    press(&mut app, KeyCode::Left);
    assert!(
        !drawn_at(&app, 120, 30).contains("THE SELECTED ROW"),
        "then the panel goes away, with the branch still open"
    );
    assert!(drawn_at(&app, 120, 30).contains('\u{25be}'), "still open");

    press(&mut app, KeyCode::Left);
    let folded = drawn_at(&app, 120, 30);
    assert!(
        !folded.contains('\u{25be}'),
        "and only then does the branch fold away: {folded}"
    );
}

#[test]
fn a_socket_under_an_open_branch_folds_it_and_lands_the_cursor_on_the_heading() {
    let mut app = on_the_sockets(BY_PROGRAM);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Left);

    let page = drawn_at(&app, 120, 30);
    assert!(
        !page.contains('\u{25be}'),
        "the branch the cursor was inside is the one that folds: {page}"
    );
    assert!(
        page.lines()
            .any(|line| line.contains(" > ") && line.contains('\u{25b8}')),
        "and the cursor is left on the heading it came out of, not on a row that moved: {page}"
    );
}
