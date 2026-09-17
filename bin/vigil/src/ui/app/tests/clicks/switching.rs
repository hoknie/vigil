use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::clicks::MOUSE;
use crate::ui::app::tests::harness::{
    app, asked, click, drawn_at, into, press, started, where_it_says,
};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

const WIDE: (u16, u16) = (120, 40);

fn on_a_marked_socket() -> App {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Char('S'));
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

#[test]
fn a_sheet_opened_to_be_copied_lets_the_mouse_go_and_takes_it_back_when_it_closes() {
    let mut app = on_a_marked_socket();

    assert!(
        app.paper.as_ref().is_some_and(|sheet| sheet.copyable),
        "the suppressions are on the screen to be copied off it"
    );
    assert!(
        !app.wants_the_mouse(),
        "a terminal whose selection this console has taken cannot copy the block it was just \
         handed, and Terminal.app has no key that gets around it"
    );
    assert_eq!(app.mouse_said(), "mouse let go");

    press(&mut app, KeyCode::Esc);

    assert!(app.paper.is_none(), "the sheet is closed");
    assert!(
        app.wants_the_mouse(),
        "and the console takes the clicks back the moment the sheet is gone"
    );
}

#[test]
fn the_m_key_turns_the_mouse_off_and_on_and_the_status_line_says_which_it_is() {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    assert!(app.wants_the_mouse());
    assert!(drawn_at(&app, WIDE.0, WIDE.1).contains("mouse on"));

    press(&mut app, KeyCode::Char(MOUSE));

    let page = drawn_at(&app, WIDE.0, WIDE.1);
    assert!(!app.wants_the_mouse(), "the mouse is off: {page}");
    assert!(
        page.contains("mouse off"),
        "a reader whose clicks stopped working has to be able to read why: {page}"
    );

    let before = app.panes().map(|panes| panes.at());
    let (column, row) = where_it_says(&page, "sshd");
    click(&mut app, column, row);
    assert_eq!(
        app.panes().map(|panes| panes.at()),
        before,
        "a console with the mouse off answers no click at all: {page}"
    );

    press(&mut app, KeyCode::Char(MOUSE));
    assert!(app.wants_the_mouse(), "and m gives it back");
}

#[test]
fn the_m_key_is_a_letter_typed_into_a_search_box_and_not_a_switch() {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Char('/'));
    press(&mut app, KeyCode::Char(MOUSE));

    assert!(
        app.wants_the_mouse(),
        "a reader searching for a word with an m in it is not asking for the mouse to go away"
    );
    assert!(
        drawn_at(&app, WIDE.0, WIDE.1).contains("m"),
        "and the letter reached the box it was typed into"
    );
}

#[test]
fn the_console_starts_without_the_mouse_when_it_is_asked_to() {
    let without = started(
        &["ui", "--socket", "/nonexistent/vigil.sock", "--no-mouse"],
        Screen::HOME,
    );
    let with = started(&["ui", "--socket", "/nonexistent/vigil.sock"], Screen::HOME);

    assert!(asked(&["ui", "--no-mouse"]).no_mouse);
    assert!(!asked(&["ui"]).no_mouse, "the mouse is on unless asked");
    assert!(
        !without.wants_the_mouse(),
        "--no-mouse is for a terminal or a person the mouse gets in the way of, and it has to \
         hold from the first frame rather than from the first m"
    );
    assert!(with.wants_the_mouse());
}

#[test]
fn a_click_that_arrives_before_the_screen_was_drawn_again_lands_on_nothing() {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "sshd");
    let before = app.panes().map(|panes| panes.at());

    app.resized();
    click(&mut app, column, row);

    assert_eq!(
        app.panes().map(|panes| panes.at()),
        before,
        "the places the console knows belong to a screen of the old size, and a click read \
         against them presses whatever used to be at that cell: {page}"
    );

    drawn_at(&app, WIDE.0, WIDE.1);
    click(&mut app, column, row);
    assert_ne!(
        app.panes().map(|panes| panes.at()),
        before,
        "once the new screen is drawn the same click is answered"
    );
}

#[test]
fn a_click_before_the_first_frame_of_a_new_size_does_not_even_close_an_open_band() {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Char('s'));
    drawn_at(&app, WIDE.0, WIDE.1);

    app.resized();
    click(&mut app, WIDE.0 - 2, WIDE.1 - 6);

    assert!(
        app.choosing(),
        "a click read against a screen that is no longer there is a click about nothing, and \
         closing the band on it would throw away what the reader was in the middle of"
    );

    drawn_at(&app, WIDE.0, WIDE.1);
    click(&mut app, WIDE.0 - 2, WIDE.1 - 6);
    assert!(
        !app.choosing(),
        "the next frame makes clicks mean something again"
    );
}

#[test]
fn a_console_that_has_drawn_nothing_yet_answers_no_click() {
    let mut app = started(&["ui", "--socket", "/nonexistent/vigil.sock"], Screen::HOME);
    app.resized();
    let level = app.level;
    let at = app.nav.sections.at();

    click(&mut app, 4, 4);

    assert_eq!(app.level, level);
    assert_eq!(
        app.nav.sections.at(),
        at,
        "nothing was drawn, so there is nothing on the screen to have been clicked"
    );
    assert_eq!(app.targets_drawn(), 0);
    assert_eq!(app.nav.at(), Screen::HOME, "and nowhere it could have gone");
}

#[test]
fn every_click_target_of_a_screen_is_somewhere_the_keys_can_reach_as_well() {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    drawn_at(&app, WIDE.0, WIDE.1);

    assert!(
        app.targets_drawn() > 0,
        "a drawn screen records what can be clicked on it"
    );
    press(&mut app, KeyCode::Char(MOUSE));
    drawn_at(&app, WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);

    assert_eq!(
        app.level,
        Level::Detail,
        "with the mouse switched off the keys still walk the whole console, because the mouse \
         never became the only way to reach anything"
    );
}
