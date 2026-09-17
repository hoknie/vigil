use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use tui_input::InputRequest;
use vigil_model::Changing;
use vigil_view::{Field, Form};

use super::drawing::{drawn_with, editing};
use crate::ui::screens::form::render;
use crate::ui::{Aim, Editing, Screen, Spot, fixture};

const LONG: &str = "/usr/local/libexec/a/very/deep/tree/of/folders/ending/in/the-shell";

fn long() -> Editing {
    Editing::open(
        Form::new("EDIT THE ACCOUNT deploy").with(Field::text("shell", "shell", LONG)),
        Screen::FINDINGS,
        0,
        Some("account|deploy".into()),
        Changing::Update,
    )
}

fn field_of(aims: &[(Aim, Rect)], spot: Spot) -> Rect {
    aims.iter()
        .find(|(aim, _)| *aim == Aim::Spot(spot))
        .map(|(_, area)| *area)
        .expect("the field is drawn and its place recorded")
}

#[test]
fn the_focused_text_field_puts_the_terminal_cursor_just_after_what_is_typed() {
    let editing = editing();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    let (cursor, aims) = render(&editing, fixture::look(), buffer.area, &mut buffer);

    let shell = field_of(&aims, Spot::Field(1));
    let cursor = cursor.expect("a text field in focus shows a real cursor, not a drawn one");
    assert_eq!(cursor.y, shell.y, "the cursor is on the field's own line");
    assert_eq!(
        cursor.x,
        shell.x + 2 + "/bin/bash".len() as u16,
        "it stands after the last letter, where the next one lands"
    );
    assert_eq!(buffer[(cursor.x - 1, cursor.y)].symbol(), "h");
    assert_eq!(
        buffer[(shell.x, shell.y)].symbol(),
        "\u{2595}",
        "the place recorded for the field starts at its left edge"
    );
}

#[test]
fn text_wider_than_the_field_scrolls_so_the_cursor_is_always_inside_it() {
    let mut editing = long();

    let (page, cursor, aims) = drawn_with(&editing, 40, 10);
    let shell = field_of(&aims, Spot::Field(0));
    let cursor = cursor.expect("drawn");
    assert!(
        cursor.x > shell.x && cursor.x < shell.right() - 1,
        "{cursor:?} is outside {shell:?}: {page}"
    );
    assert!(
        page.contains("the-shell"),
        "the end is shown while typing at the end: {page}"
    );
    assert!(!page.contains("/usr/local"), "{page}");
    for line in page.lines() {
        assert!(line.chars().count() <= 40, "{line}");
    }

    editing.edit(InputRequest::GoToStart);
    let (page, cursor, _) = drawn_with(&editing, 40, 10);
    assert!(page.contains("\u{2595} /usr/local"), "{page}");
    assert_eq!(cursor, Some(Position::new(shell.x + 2, shell.y)), "{page}");
}

#[test]
fn a_field_out_of_focus_shows_the_start_of_its_text_and_no_cursor() {
    let mut editing = long();
    editing.previous();

    let (page, cursor, _) = drawn_with(&editing, 40, 10);

    assert_eq!(cursor, None, "{page}");
    assert!(page.contains("\u{2595} /usr/local"), "{page}");
}
