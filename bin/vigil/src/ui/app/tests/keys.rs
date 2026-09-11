use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::ui::{Level, Screen};

use super::harness::{app, drawn, into, press};

#[test]
fn the_full_list_of_keys_opens_over_the_page_and_any_key_closes_it() {
    let mut app = app();

    press(&mut app, KeyCode::Char('?'));
    assert!(drawn(&app).contains("KEYS"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('q'));
    assert!(
        !app.leaving,
        "q closes the list of keys rather than the console under it"
    );
    assert!(!drawn(&app).contains("any key closes"), "{}", drawn(&app));
}

#[test]
fn control_c_leaves_from_under_the_list_of_keys_as_well() {
    let mut app = app();
    press(&mut app, KeyCode::Char('?'));

    app.on_key(KeyCode::Char('c'), KeyModifiers::CONTROL);

    assert!(app.leaving);
}

#[test]
fn q_leaves_from_every_screen_and_every_level() {
    let mut home = app();
    press(&mut home, KeyCode::Char('q'));
    assert!(home.leaving, "q did nothing on the main screen");

    for screen in Screen::ALL {
        for depth in 0..3 {
            let mut app = app();
            into(&mut app, *screen, 200, 24);
            for _ in 0..depth {
                press(&mut app, KeyCode::Enter);
            }
            press(&mut app, KeyCode::Char('q'));
            assert!(app.leaving, "q did nothing on {}", screen.name());
        }
    }
}

#[test]
fn the_tab_key_does_nothing_now_that_there_is_no_row_of_tabs_to_walk() {
    let mut app = app();
    press(&mut app, KeyCode::Char('3'));
    let before = app.nav.at();

    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::BackTab);

    assert_eq!(app.nav.at(), before);
    assert_eq!(app.level, Level::Menu);
}
