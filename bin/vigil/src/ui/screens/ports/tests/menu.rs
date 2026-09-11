use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{Given, busy, drawn, drawn_with, showing};
use crate::ui::helpers::words::text;
use crate::ui::screens::ports::{Arrangement, Showing, render};
use crate::ui::{Arrows, Protocols, fixture};

#[test]
fn the_two_views_are_a_row_of_names_with_the_open_one_in_brackets() {
    let page = drawn(&busy(), 0, 80);
    let row = page.lines().next().expect("a submenu");

    assert!(row.contains("[sockets]"), "{row}");
    assert!(row.contains("by program"), "{row}");
    assert!(row.chars().count() <= 80, "{row}");
}

#[test]
fn the_caret_says_when_the_arrows_are_on_the_row_of_views() {
    let host = busy();
    let given = Given::default();
    let drawn = |arrows| {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 20));
        render(
            &host,
            fixture::look(),
            &Showing {
                arrows,
                ..given.showing(0)
            },
            buffer.area,
            &mut buffer,
        );
        text::to_text(&buffer)
    };

    assert!(
        drawn(Arrows::Menu).starts_with(" ▸ "),
        "{}",
        drawn(Arrows::Menu)
    );
    assert!(!drawn(Arrows::List).starts_with(" ▸ "));
    assert!(
        drawn(Arrows::List)
            .lines()
            .any(|line| line.starts_with(" > ")),
        "the cursor is in the table instead: {}",
        drawn(Arrows::List)
    );
}

#[test]
fn each_view_says_what_it_is_and_which_of_the_two_is_the_reading_itself() {
    let flat = drawn(&busy(), 0, 80);
    assert!(flat.contains("one row per listening socket"), "{flat}");
    assert!(flat.contains("keyed by"), "{flat}");

    let grouped = drawn_with(
        &busy(),
        &showing(Protocols::default(), Arrangement::ByProgram),
        0,
        80,
    );
    assert!(grouped.contains("one row per program"), "{grouped}");
    let unbroken = grouped.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(unbroken.contains("no resolved owner"), "{grouped}");
}
