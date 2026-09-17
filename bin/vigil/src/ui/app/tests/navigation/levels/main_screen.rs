use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn, into, number, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

#[test]
fn the_console_opens_on_the_main_screen_and_the_arrows_are_in_its_list() {
    let app = app();

    assert_eq!(app.nav.at(), Screen::HOME);
    assert_eq!(app.level, Level::List);
    assert!(
        drawn(&app).contains("What this agent watches"),
        "{}",
        drawn(&app)
    );
}

#[test]
fn the_main_screen_opens_the_section_the_cursor_is_on() {
    let mut app = app();
    press(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Enter);

    assert_eq!(app.nav.at(), screen("accounts"));
    assert_eq!(
        app.level,
        Level::Menu,
        "and lands on that section's top rung, which is its row of lists"
    );
}

#[test]
fn the_right_arrow_opens_a_section_the_way_enter_does() {
    let mut app = app();

    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.at(), screen("ports"));
}

#[test]
fn escape_from_the_top_of_a_section_goes_to_the_main_screen_and_nowhere_else() {
    for screen in Screen::all() {
        let mut app = app();
        press(&mut app, number(screen));
        assert_eq!(app.nav.at(), screen);

        press(&mut app, KeyCode::Esc);

        assert_eq!(
            app.nav.at(),
            Screen::HOME,
            "Escape from the top of {} went somewhere else",
            screen.name()
        );
        press(&mut app, KeyCode::Esc);
        assert!(
            !app.leaving,
            "and the main screen is the top: people leave with q"
        );
    }
}

#[test]
fn the_main_screen_holds_the_cursor_on_the_section_it_was_left_from() {
    let mut app = app();

    press(&mut app, number(screen("startup")));
    press(&mut app, KeyCode::Esc);

    let page = drawn(&app);
    assert!(
        page.lines()
            .any(|line| line.contains(" \u{25b8}   4 startup")),
        "coming back put the reader at the top of the list: {page}"
    );
}

#[test]
fn a_number_opens_its_section_from_every_rung_and_lands_on_that_section_s_top_one() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 200, 24);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, number(screen("accounts")));

    assert_eq!(app.nav.at(), screen("accounts"));
    assert_eq!(app.level, Level::Menu);

    press(&mut app, number(Screen::SUMMARY));
    assert_eq!(app.nav.at(), Screen::SUMMARY);
    assert_eq!(
        app.level,
        Level::List,
        "a section without a row of lists is entered at its list"
    );
}

#[test]
fn a_digit_leaves_the_submenu_the_way_it_leaves_every_other_level() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));
    assert_eq!(app.level, Level::Menu);

    press(&mut app, number(screen("ports")));

    assert_eq!(app.nav.at(), screen("ports"));
    assert_eq!(app.level, Level::Menu);
}

#[test]
fn a_number_with_no_section_behind_it_changes_nothing() {
    let mut app = app();
    press(&mut app, number(screen("ports")));

    press(&mut app, KeyCode::Char('0'));

    assert_eq!(app.nav.at(), screen("ports"));
}
