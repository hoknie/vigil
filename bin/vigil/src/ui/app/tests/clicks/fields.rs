use ratatui::crossterm::event::KeyCode;

use crate::ui::Spot;
use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, click, drawn_at, into, press, where_it_says};
use crate::ui::fixture::screen;

const NARROW: (u16, u16) = (80, 24);

fn on_the_deploy_form() -> App {
    let mut app = app();
    into(&mut app, screen("accounts"), NARROW.0, NARROW.1);
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
    drawn_at(&app, NARROW.0, NARROW.1);
    app
}

fn with_the_groups_open() -> App {
    let mut app = on_the_deploy_form();
    for _ in 0..20 {
        if app
            .editing
            .as_ref()
            .is_some_and(|editing| editing.in_a_list())
        {
            break;
        }
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Enter);
    drawn_at(&app, NARROW.0, NARROW.1);
    app
}

fn chosen(app: &App) -> Vec<String> {
    app.editing
        .as_ref()
        .map(|editing| {
            editing
                .choices()
                .iter()
                .filter(|choice| choice.chosen)
                .map(|choice| choice.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_click_on_a_field_of_a_form_puts_the_focus_in_that_field() {
    let mut app = on_the_deploy_form();
    let page = drawn_at(&app, NARROW.0, NARROW.1);
    let was = app.editing.as_ref().map(crate::ui::Editing::spot);
    let (column, row) = where_it_says(&page, "/home/deploy");

    click(&mut app, column, row);
    let now = app.editing.as_ref().map(crate::ui::Editing::spot);

    assert_ne!(now, was, "the click chose another field: {page}");
    assert!(
        matches!(now, Some(Spot::Field(_))),
        "a field pointed at is a field being edited, with the terminal's cursor in it: {now:?}"
    );
    assert!(
        drawn_at(&app, NARROW.0, NARROW.1).contains("\u{25b8} home"),
        "and the mark that says where the keys are moved with it: {}",
        drawn_at(&app, NARROW.0, NARROW.1)
    );
}

#[test]
fn a_click_on_a_row_of_the_dropdown_turns_that_choice_on_and_off_as_space_does() {
    let mut app = with_the_groups_open();
    let page = drawn_at(&app, NARROW.0, NARROW.1);
    let was = chosen(&app);
    let (column, row) = where_it_says(&page, "[ ] users");

    click(&mut app, column + 2, row);
    let after = chosen(&app);

    assert!(
        after.contains(&"users".to_string()),
        "the row clicked is the row toggled: {page}"
    );
    assert!(
        app.editing
            .as_ref()
            .is_some_and(|editing| editing.dropdown().is_some()),
        "and the list stays open, the way Space leaves it open, so a reader can pick another"
    );

    let page = drawn_at(&app, NARROW.0, NARROW.1);
    let (column, row) = where_it_says(&page, "[x] users");
    click(&mut app, column + 2, row);

    assert_eq!(
        chosen(&app),
        was,
        "clicking a chosen row takes the choice back: {page}"
    );
}

#[test]
fn a_click_beside_the_open_dropdown_closes_it_and_keeps_what_was_toggled() {
    let mut app = with_the_groups_open();
    let page = drawn_at(&app, NARROW.0, NARROW.1);
    let (column, row) = where_it_says(&page, "[ ] users");
    click(&mut app, column + 2, row);
    let kept = chosen(&app);

    click(&mut app, 2, NARROW.1 - 5);

    assert!(
        app.editing
            .as_ref()
            .is_some_and(|editing| editing.dropdown().is_none()),
        "a click outside the list closes it, as Esc does: {page}"
    );
    assert!(app.editing.is_some(), "and the form itself stays open");
    assert_eq!(
        chosen(&app),
        kept,
        "with the toggles it was given, because Esc keeps them too"
    );
}

#[test]
fn a_click_on_the_back_button_of_a_form_leaves_the_form() {
    let mut app = on_the_deploy_form();
    let page = drawn_at(&app, NARROW.0, NARROW.1);
    let (column, row) = where_it_says(&page, "[ \u{2190} Back ]");

    click(&mut app, column + 3, row);

    assert!(
        app.editing.is_none(),
        "the button says it goes back and a click on it goes back: {page}"
    );
}
