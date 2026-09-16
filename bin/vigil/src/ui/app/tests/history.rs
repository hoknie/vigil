use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::ui::app::App;
use crate::ui::app::deeds::{DELETE, EDIT, KILL, MARK, NEW, SUPPRESS, UNMARK_EVERY};
use crate::ui::app::history::HISTORY;
use crate::ui::app::narrowing::DETAILS;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;
use crate::ui::helpers::motion::keys;
use crate::ui::{Action, Level, Screen, holding};

const RUNNING: usize = 0;

const LAUNCHES: usize = 1;

const NC: &str = "run|alice|/usr/bin/nc.openbsd";

fn on_the_programs(pane: usize) -> App {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    press(&mut app, number(screen("programs")));
    drawn_at(&app, 120, 30);
    for _ in 0..pane {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    app
}

fn on_nc() -> App {
    let mut app = on_the_programs(LAUNCHES);
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == NC)
        {
            return app;
        }
        press(&mut app, KeyCode::Down);
    }
    panic!("{NC} is not on the list: {:?}", app.pane_keys());
}

#[test]
fn h_opens_the_history_of_the_launch_under_the_cursor_in_place_of_the_list_and_esc_comes_back() {
    let mut app = on_nc();

    press(&mut app, KeyCode::Char(HISTORY));
    let page = drawn_at(&app, 120, 30);

    assert!(app.history.is_some(), "H opened nothing on {NC}");
    assert!(page.contains("THE HISTORY OF THE SELECTED ROW"), "{page}");
    assert!(
        page.contains("RUNS OF") && page.contains("/usr/bin/nc.openbsd"),
        "the panel is about the row under the cursor and names it: {page}"
    );
    assert!(page.contains("1757419207.000:3425"), "{page}");
    assert!(
        !page.contains("FIRST SEEN"),
        "the history is a panel of its own, not a box drawn over the list: {page}"
    );
    assert!(page.contains("Esc back to the list"), "{page}");

    press(&mut app, KeyCode::Esc);
    let back = drawn_at(&app, 120, 30);
    assert!(app.history.is_none());
    assert!(back.contains("FIRST SEEN"), "{back}");
    assert_eq!(
        app.pane_row_under_the_cursor()
            .map(|row| row.key)
            .as_deref(),
        Some(NC),
        "the cursor is where the reader left it"
    );
}

#[test]
fn the_left_arrow_leaves_the_history_as_it_goes_back_everywhere_else() {
    let mut app = on_nc();
    press(&mut app, KeyCode::Char(HISTORY));

    press(&mut app, KeyCode::Left);

    assert!(app.history.is_none());
}

#[test]
fn a_letter_pressed_in_the_history_does_nothing_to_the_list_underneath() {
    let mut app = on_nc();
    press(&mut app, KeyCode::Char(HISTORY));

    for letter in [MARK, KILL, SUPPRESS, 'f', 's'] {
        press(&mut app, KeyCode::Char(letter));
    }

    assert!(app.history.is_some(), "only a way back closes the panel");
    assert!(app.marked().is_empty(), "{:?}", app.marked());
    assert_eq!(app.chooser.choosing(), None);
    assert!(app.paper.is_none());
}

#[test]
fn a_list_that_keeps_no_history_says_so_when_h_is_pressed() {
    let mut app = on_the_programs(RUNNING);

    press(&mut app, KeyCode::Char(HISTORY));

    assert!(app.history.is_none());
    assert!(
        drawn_at(&app, 160, 30).contains("keeps no history"),
        "{}",
        drawn_at(&app, 160, 30)
    );
}

#[test]
fn the_launches_offer_h_at_the_foot_of_the_screen_even_at_eighty_columns() {
    let app = on_nc();

    for width in [80u16, 160] {
        let page = drawn_at(&app, width, 30);
        assert!(page.contains("H history"), "{width}: {page}");
    }
    let running = on_the_programs(RUNNING);
    assert!(
        !drawn_at(&running, 160, 30).contains("H history"),
        "a key the list answers with a refusal is a key the line must not offer"
    );
}

#[test]
fn no_list_and_no_deed_answers_to_the_key_that_opens_a_history_with_something_else() {
    assert!(
        ![
            MARK,
            UNMARK_EVERY,
            SUPPRESS,
            NEW,
            EDIT,
            DELETE,
            KILL,
            DETAILS
        ]
        .contains(&HISTORY),
        "{HISTORY} is already a deed"
    );
    for shift in [KeyModifiers::NONE, KeyModifiers::SHIFT] {
        assert_eq!(
            keys::action(KeyCode::Char(HISTORY), shift, false),
            Action::Letter(HISTORY),
            "a terminal that reports the shift beside the capital must not mean something else"
        );
    }
    for screen in Screen::all() {
        let Some(section) = holding(screen.name()) else {
            continue;
        };
        for pane in section.panes() {
            assert!(
                pane.toggles().iter().all(|toggle| toggle.key != HISTORY)
                    && pane.arrangements().iter().all(|one| one.key != HISTORY),
                "{} switches something on {HISTORY}, and the history would never open there",
                pane.name()
            );
        }
    }
}
