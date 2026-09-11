use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{drawn, drawn_body, quiet};
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::frame::{Hints, render};
use crate::ui::helpers::words::text;
use crate::ui::{Level, Screen, fixture};

fn drawn_with(hints: Hints<'_>) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 24));
    render(
        fixture::look(),
        Screen::Findings,
        &fixture::view(),
        hints,
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn the_key_hints_change_with_the_level_and_with_the_search_box() {
    let list = drawn_with(Hints {
        level: Level::List,
        ..quiet()
    });
    assert!(list.contains("/ search"), "{list}");
    assert!(list.contains("→ detail"), "{list}");

    let typing = drawn_with(Hints {
        typing: true,
        ..quiet()
    });
    assert!(typing.contains("type to search"), "{typing}");
    assert!(
        !typing.contains("q quit"),
        "q is a letter in here: {typing}"
    );
}

#[test]
fn the_hint_names_the_place_escape_goes_back_to_and_changes_when_that_place_does() {
    let plain = drawn_with(Hints {
        level: Level::List,
        ..quiet()
    });
    let after_a_jump = drawn_with(Hints {
        level: Level::List,
        back: Back::Finding,
        ..quiet()
    });

    assert!(plain.contains("back to the main screen"), "{plain}");
    assert!(
        after_a_jump.contains("back to the finding"),
        "{after_a_jump}"
    );
}

#[test]
fn a_terminal_with_no_room_for_the_long_hint_still_says_where_the_rest_are() {
    let (page, _) = drawn(fixture::look(), Screen::Findings, &fixture::view(), 40, 24);

    assert!(page.contains("? keys"), "{page}");
    for line in page.lines() {
        assert!(line.chars().count() <= 40, "{line}");
    }
}

#[test]
fn an_answer_to_the_key_just_pressed_takes_the_hint_bar_and_does_not_move_the_table() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 24));
    let body = render(
        fixture::look(),
        Screen::Findings,
        &fixture::view(),
        Hints {
            message: Some("That socket is not in the current reading."),
            ..quiet()
        },
        buffer.area,
        &mut buffer,
    );
    let with_message = text::to_text(&buffer);

    assert!(
        with_message.contains("not in the current reading"),
        "{with_message}"
    );
    assert!(!with_message.contains("q quit"), "{with_message}");
    assert_eq!(body, drawn_body(), "the screen keeps every row it had");
}
