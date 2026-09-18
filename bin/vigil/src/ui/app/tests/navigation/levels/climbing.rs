use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, into, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

fn opened_on(screen: Screen) -> App {
    let mut app = app();
    into(&mut app, screen, 120, 40);
    assert_eq!(
        app.level,
        Level::List,
        "{} did not open on a list to climb out of",
        screen.name()
    );
    app
}

#[test]
fn the_arrow_up_from_the_first_row_of_any_section_climbs_out_to_the_main_screen() {
    for screen in Screen::all() {
        let mut app = opened_on(screen);

        for press_number in 1..=6 {
            press(&mut app, KeyCode::Up);
            if app.nav.at() == Screen::HOME {
                break;
            }
            assert!(
                press_number < 6,
                "{} holds the reader on its first row: the arrows are the way out of a \
                 section as much as the way down it, and a screen that stops the cursor at \
                 the top is a screen with no way back that does not need another key",
                screen.name()
            );
        }

        assert_eq!(
            app.nav.at(),
            Screen::HOME,
            "{} never reached the main screen",
            screen.name()
        );
    }
}

#[test]
fn escape_leaves_every_section_and_the_left_arrow_leaves_the_ones_with_no_row_of_names() {
    for screen in Screen::all() {
        let mut app = opened_on(screen);

        for _ in 0..6 {
            press(&mut app, KeyCode::Esc);
            if app.nav.at() == Screen::HOME {
                break;
            }
        }

        assert_eq!(
            app.nav.at(),
            Screen::HOME,
            "Esc does not leave {}",
            screen.name()
        );
    }

    for screen in Screen::all() {
        let mut app = opened_on(screen);
        let along_a_row_of_names = app.rungs().menu || app.rungs().groups;

        for _ in 0..6 {
            press(&mut app, KeyCode::Left);
            if app.nav.at() == Screen::HOME {
                break;
            }
        }

        match along_a_row_of_names {
            true => assert_eq!(
                app.nav.at(),
                screen,
                "on a section with a row of names ← walks the names, and ↑ or Esc is the way \
                 out: {} left under a key that means something else there",
                screen.name()
            ),
            false => assert_eq!(
                app.nav.at(),
                Screen::HOME,
                "{} has no row of names, so ← means what Esc means and has to leave it",
                screen.name()
            ),
        }
    }
}

#[test]
fn a_panel_opened_on_a_row_is_put_away_before_the_section_is_and_never_holds_the_reader() {
    for screen in Screen::all() {
        let mut app = opened_on(screen);
        press(&mut app, KeyCode::Right);

        for _ in 0..6 {
            press(&mut app, KeyCode::Up);
            if app.nav.at() == Screen::HOME {
                break;
            }
        }

        assert_eq!(
            app.nav.at(),
            Screen::HOME,
            "{} keeps the reader once a detail has been opened on it",
            screen.name()
        );
        assert!(!app.detail_open, "{} left a panel open", screen.name());
    }
}

#[test]
fn the_main_screen_is_the_top_and_the_arrow_up_on_its_first_row_goes_nowhere() {
    let mut app = app();
    drawn_at(&app, 120, 40);

    press(&mut app, KeyCode::Up);

    assert_eq!(app.nav.at(), Screen::HOME);
    assert!(
        !app.leaving,
        "the way out of the console is q, not the arrows"
    );
}

#[test]
fn no_screen_hands_the_arrows_to_the_panel_on_the_first_press() {
    for screen in [
        Screen::FINDINGS,
        screen("network"),
        screen("accounts"),
        screen("programs"),
        screen("startup"),
    ] {
        let mut app = app();
        into(&mut app, screen, 160, 30);

        press(&mut app, KeyCode::Right);

        assert_eq!(
            app.level,
            Level::List,
            "{}: the panel opens and the arrows stay where the reader put them. One rule on \
             every screen, or a reader learns one screen and is surprised by the next",
            screen.name()
        );
        assert!(app.detail_open, "{}: and the panel did open", screen.name());

        press(&mut app, KeyCode::Right);
        assert_eq!(
            app.level,
            Level::Detail,
            "{}: the second press hands them over",
            screen.name()
        );
    }
}

#[test]
fn a_terminal_with_no_room_beside_the_list_keeps_the_one_press_it_always_had() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 24);

    press(&mut app, KeyCode::Right);

    assert_eq!(
        app.level,
        Level::Detail,
        "there is nowhere to put a panel beside the list here, so a rung that shows nothing \
         would be a press that does nothing"
    );
}
