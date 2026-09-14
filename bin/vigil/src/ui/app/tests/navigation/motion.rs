use ratatui::crossterm::event::KeyCode;
use vigil_model::Severity;

use crate::ui::Screen;
use crate::ui::fixture;

use crate::ui::app::tests::harness::{app, into, press};
use crate::ui::fixture::screen;

#[test]
fn the_cursor_holds_on_to_its_finding_when_a_new_one_arrives_above_it() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);
    press(&mut app, KeyCode::Down);
    let held = app
        .selected_finding()
        .expect("something is selected")
        .title
        .clone();

    app.view.found.findings.insert(
        0,
        fixture::finding("something that has just happened", Severity::High),
    );
    app.settle();

    assert_eq!(
        app.selected_finding().expect("still selected").title,
        held,
        "the cursor followed its finding rather than staying on row two"
    );
}

#[test]
fn a_list_that_shrank_under_the_cursor_does_not_leave_it_pointing_past_the_end() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);
    press(&mut app, KeyCode::Down);

    app.view.found.findings.truncate(1);
    app.settle();

    assert_eq!(app.nav.findings.at(), 0);
    assert!(app.selected_finding().is_some());
}

#[test]
fn the_page_keys_and_the_ends_of_a_list_all_work() {
    let mut app = app();
    for number in 0..40 {
        app.view.found.findings.push(fixture::finding(
            &format!("finding number {number}"),
            Severity::Low,
        ));
    }
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::End);
    assert_eq!(app.nav.findings.at(), app.view.found.findings.len() - 1);

    press(&mut app, KeyCode::Home);
    assert_eq!(app.nav.findings.at(), 0);

    press(&mut app, KeyCode::PageDown);
    assert!(
        app.nav.findings.at() > 1,
        "a page key that moves one row is a page key that does nothing"
    );
}

#[test]
fn the_main_screen_keeps_the_cursor_on_the_section_it_was_left_from() {
    let mut app = app();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.nav.at(), screen("programs"));

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::HOME);
    assert_eq!(
        app.nav.sections.at(),
        2,
        "coming out of a section put the reader back at the top of the main screen"
    );
}

#[test]
fn each_list_keeps_the_row_its_reader_left_it_on() {
    let mut app = app();
    into(&mut app, screen("accounts"), 120, 40);
    press(&mut app, KeyCode::Down);
    let was = app.pane_keys()[app.panes().expect("a section").at()].clone();

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Left);

    assert_eq!(app.panes().expect("a section").showing(), 0);
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        was,
        "coming back to a list put the reader at the top of it"
    );
}
