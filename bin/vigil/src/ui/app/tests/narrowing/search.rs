use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn, press, typed};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

#[test]
fn a_search_narrows_the_findings_and_the_letters_do_not_reach_the_console() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::FINDINGS));

    press(&mut app, KeyCode::Char('/'));
    assert_eq!(
        app.level,
        Level::List,
        "the search takes the arrows into the list"
    );

    typed(&mut app, "logged in q");
    assert!(!app.leaving, "the q was a letter, not the quit key");

    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);

    let page = drawn(&app);
    assert!(page.contains("A user logged in"), "{page}");
    assert!(!page.contains("A new listening port"), "{page}");
}

#[test]
fn escape_takes_a_search_off_before_it_takes_the_screen_away() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::FINDINGS));
    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "nothing matches this");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::List);
    assert!(app.filter.holding_back());

    press(&mut app, KeyCode::Esc);
    assert!(!app.filter.holding_back(), "the search went first");
    assert_eq!(app.level, Level::List, "and the arrows stayed on the list");
    assert!(
        drawn(&app).contains("A new listening port"),
        "{}",
        drawn(&app)
    );

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::Menu, "then up to the row of lists");

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::HOME, "then out of the section");

    press(&mut app, KeyCode::Esc);
    assert!(!app.leaving, "and the main screen is the top");
}

#[test]
fn the_search_opens_on_the_screen_the_reader_is_on_and_never_moves_them() {
    for screen in [screen("network"), screen("accounts"), Screen::FINDINGS] {
        let mut app = app();
        press(&mut app, super::harness::number(screen));
        assert_ne!(
            app.level,
            Level::Detail,
            "the arrows start above the detail"
        );

        press(&mut app, KeyCode::Char('/'));

        assert_eq!(app.nav.at(), screen, "the screen changed under the reader");
        assert_eq!(
            app.level,
            Level::List,
            "and the arrows went into this screen's list"
        );
        assert!(app.typing(), "{} did not open its box", screen.name());
    }
}

#[test]
fn on_a_screen_with_no_list_the_search_says_so_rather_than_moving_the_reader() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::SUMMARY));

    press(&mut app, KeyCode::Char('/'));

    assert_eq!(app.nav.at(), Screen::SUMMARY);
    assert!(!app.typing());
    assert!(
        drawn(&app).contains("Nothing to search on this screen"),
        "{}",
        drawn(&app)
    );
}

#[test]
fn the_main_screen_has_no_search_and_says_where_the_search_lives() {
    let mut app = app();

    press(&mut app, KeyCode::Char('/'));

    assert_eq!(
        app.nav.at(),
        Screen::HOME,
        "and it does not move the reader"
    );
    assert!(!app.typing());
    assert!(
        drawn(&app).contains("open one and press /"),
        "{}",
        drawn(&app)
    );
}
