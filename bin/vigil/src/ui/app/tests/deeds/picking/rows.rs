use ratatui::crossterm::event::KeyCode;

use super::findings::{control, on_the_findings, shift};
use crate::ui::Deed;
use crate::ui::app::tests::harness::{drawn, press};

#[test]
fn a_console_nobody_has_picked_anything_on_draws_no_bar_and_no_gutter_for_one() {
    let app = on_the_findings();

    let page = drawn(&app);

    assert!(!page.contains("picked"), "{page}");
    assert!(
        page.contains("A new listening port"),
        "the table is drawn as it always was: {page}"
    );
}

#[test]
fn shift_with_an_arrow_picks_the_rows_it_walks_over_and_says_how_many() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Down);

    let page = drawn(&app);
    assert!(page.contains("2 picked"), "{page}");
    assert!(
        page.contains(Deed::Remove.named()),
        "a bar with no deed on it is a bar that does nothing: {page}"
    );
}

#[test]
fn a_picked_row_is_marked_with_a_character_and_not_only_a_colour() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Down);

    let page = drawn(&app);
    let marked = page.lines().filter(|line| line.contains('\u{d7}')).count();
    assert_eq!(
        marked, 2,
        "a reader with no colour has to see which two rows a deed would reach: {page}"
    );
}

#[test]
fn control_with_an_arrow_gathers_rows_that_are_not_beside_each_other() {
    let mut app = on_the_findings();

    control(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    control(&mut app, KeyCode::Down);

    let page = drawn(&app);
    assert!(page.contains("2 picked"), "{page}");
    let lines: Vec<&str> = page
        .lines()
        .filter(|line| line.contains("09:00:00"))
        .collect();
    assert!(
        lines[0].contains('\u{d7}') && !lines[1].contains('\u{d7}') && lines[2].contains('\u{d7}'),
        "the row between the two picked ones was picked as well: {page}"
    );
}

#[test]
fn the_key_under_the_cursor_picks_one_row_where_a_terminal_sends_no_modifiers() {
    let mut app = on_the_findings();

    press(&mut app, KeyCode::Char('x'));

    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('x'));

    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}

#[test]
fn a_whole_screenful_goes_on_and_comes_off_with_the_same_key() {
    let mut app = on_the_findings();

    press(&mut app, KeyCode::Char('a'));
    assert!(drawn(&app).contains("3 picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('a'));
    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}
