use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, into, press};
use crate::ui::fixture::screen;
use crate::ui::helpers::words::text;
use crate::ui::{Choosing, Screen};

fn on_the_deploy_form() -> App {
    let mut app = app();
    into(&mut app, screen("accounts"), 80, 24);
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == "account|deploy")
        {
            break;
        }
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Char('e'));
    assert!(app.editing.is_some(), "the form opened");
    app
}

fn painted(app: &App) -> (Buffer, String) {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(buffer.area, &mut buffer);
    let page = text::to_text(&buffer);
    (buffer, page)
}

#[test]
fn the_console_hands_the_terminal_a_cursor_on_the_field_being_typed_in_and_on_nothing_else() {
    let mut app = on_the_deploy_form();

    let (buffer, page) = painted(&app);
    let cursor = app.cursor().expect("a text field has the focus");
    assert_eq!(buffer[(cursor.x - 1, cursor.y)].symbol(), "h", "{page}");
    assert!(
        page.lines()
            .nth(cursor.y as usize)
            .is_some_and(|line| line.contains("shell"))
    );

    press(&mut app, KeyCode::Char('w'));
    press(&mut app, KeyCode::Char('w'));
    press(&mut app, KeyCode::Char('w'));
    press(&mut app, KeyCode::Char('h'));
    app.on_key(KeyCode::Char('w'), KeyModifiers::CONTROL);
    let editing = app
        .editing
        .as_ref()
        .expect("a chord in a field does not close the form");
    assert_eq!(editing.form().text("shell"), Some("/bin/"));

    press(&mut app, KeyCode::Up);
    painted(&app);
    assert_eq!(
        app.cursor(),
        None,
        "a button has no text to put a cursor in"
    );
}

#[test]
fn the_groups_open_as_a_list_over_the_form_on_an_eighty_by_twenty_four_terminal() {
    let mut app = on_the_deploy_form();
    while !app
        .editing
        .as_ref()
        .is_some_and(|editing| editing.in_a_list())
    {
        press(&mut app, KeyCode::Down);
    }
    let (_, closed) = painted(&app);
    assert!(closed.contains("\u{2595} wheel"), "{closed}");
    assert!(
        closed.contains("Enter a list"),
        "the key line says how it opens: {closed}"
    );

    press(&mut app, KeyCode::Enter);
    let (_, open) = painted(&app);
    assert!(
        open.contains("[x] wheel") && open.contains("[ ] root"),
        "{open}"
    );
    assert!(open.contains("toggles kept"), "{open}");
    assert_eq!(app.cursor(), None);
    assert!(open.lines().count() <= 24);
    for line in open.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }

    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Esc);
    let editing = app
        .editing
        .as_ref()
        .expect("Esc closed the list, and the form is still open");
    assert_eq!(editing.form().chosen("groups"), Some(vec!["root", "wheel"]));
    assert!(
        editing.form().changed("groups"),
        "the toggle made in the list was kept"
    );
}

#[test]
fn a_popup_of_options_is_walked_by_the_up_and_down_arrows_and_still_by_the_side_ones() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 24);
    press(&mut app, KeyCode::Char('s'));
    assert_eq!(app.chooser.choosing(), Some(Choosing::Sort));
    let start = app.chooser.at();

    press(&mut app, KeyCode::Down);
    assert_eq!(app.chooser.at(), start + 1);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.chooser.at(), start + 2);
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.chooser.at(), start);

    let page = drawn_at(&app, 80, 24);
    assert!(page.contains("\u{25b8} as the agent sends it"), "{page}");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.chooser.choosing(), None);
    assert!(
        !drawn_at(&app, 80, 24).contains("sort by"),
        "a closed popup leaves nothing behind"
    );
}
