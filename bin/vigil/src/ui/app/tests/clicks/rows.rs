use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, click, drawn_at, into, press, where_it_says};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

const WIDE: (u16, u16) = (120, 40);

fn on_the_ports() -> App {
    let mut app = app();
    into(&mut app, screen("network"), WIDE.0, WIDE.1);
    app
}

fn row_under_the_cursor(app: &App) -> String {
    app.pane_row_under_the_cursor()
        .map(|row| row.key)
        .unwrap_or_default()
}

#[test]
fn a_click_on_a_row_puts_the_cursor_on_it_and_a_second_click_opens_it_as_the_arrow_does() {
    let mut app = on_the_ports();
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let first = row_under_the_cursor(&app);
    let (column, row) = where_it_says(&page, "sshd");

    click(&mut app, column, row);
    drawn_at(&app, WIDE.0, WIDE.1);
    let clicked = row_under_the_cursor(&app);

    assert_ne!(
        clicked, first,
        "the cursor stayed where it was although the reader pointed at another row: {page}"
    );
    assert_eq!(
        app.level,
        Level::List,
        "one click chooses a row and does not walk into it"
    );

    click(&mut app, column, row);
    drawn_at(&app, WIDE.0, WIDE.1);

    assert!(
        app.detail_open,
        "a second click on the row already under the cursor is the \u{2192} of that row, which \
         opens what that row holds: {}",
        drawn_at(&app, WIDE.0, WIDE.1)
    );
    assert_eq!(
        row_under_the_cursor(&app),
        clicked,
        "and it opens the row that was clicked, not the one the list had before"
    );
}

#[test]
fn what_a_click_on_a_row_does_is_what_the_arrows_and_the_arrow_key_do() {
    let mut clicked = on_the_ports();
    let page = drawn_at(&clicked, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "sshd");
    click(&mut clicked, column, row);
    let wanted = row_under_the_cursor(&clicked);

    let mut pressed = on_the_ports();
    for _ in 0..40 {
        if row_under_the_cursor(&pressed) == wanted {
            break;
        }
        press(&mut pressed, KeyCode::Down);
    }
    click(&mut clicked, column, row);
    press(&mut pressed, KeyCode::Right);

    assert_eq!(pressed.level, clicked.level);
    assert_eq!(
        row_under_the_cursor(&pressed),
        row_under_the_cursor(&clicked),
        "the mouse and the keyboard have to leave the console in the same state, or one of them \
         is a second way to be somewhere the other cannot reach"
    );
    assert_eq!(pressed.detail_open, clicked.detail_open);
}

#[test]
fn a_click_on_the_name_of_another_list_of_the_section_opens_that_list() {
    let mut app = app();
    into(&mut app, screen("network"), WIDE.0, WIDE.1);
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let showing = app.panes().map(|panes| panes.showing()).unwrap_or_default();
    let (column, row) = where_it_says(&page, "by program");

    click(&mut app, column + 1, row);
    drawn_at(&app, WIDE.0, WIDE.1);

    assert_ne!(
        app.panes().map(|panes| panes.showing()),
        Some(showing),
        "the row of list names is a row of places to click, and a click on one of them opens \
         the list it names: {page}"
    );
    assert_eq!(
        app.pane().map(|pane| pane.name().to_string()),
        Some("by program".to_string())
    );
}

#[test]
fn a_click_on_a_section_of_the_main_screen_chooses_it_and_a_second_one_opens_it() {
    let mut app = app();
    let page = drawn_at(&app, 80, 24);
    let (column, row) = where_it_says(&page, "accounts");

    click(&mut app, column, row);
    drawn_at(&app, 80, 24);
    assert_eq!(
        app.nav.at(),
        Screen::HOME,
        "the first click chooses the section and stays on the main screen"
    );
    assert_eq!(
        app.section_under_cursor().map(|row| row.name),
        Some("accounts".to_string()),
        "{page}"
    );

    click(&mut app, column, row);
    assert_eq!(
        app.nav.at(),
        screen("accounts"),
        "and the second opens it, as \u{2192} does on the row the cursor is on"
    );
}
