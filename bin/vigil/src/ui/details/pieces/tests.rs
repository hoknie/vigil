use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_view::Piece;

use super::{height, render};
use crate::ui::fixture;
use crate::ui::helpers::words::text;

fn drawn(pieces: &[Piece]) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 20));
    render(pieces, fixture::look(), 0, buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn every_kind_of_piece_reaches_the_screen_as_the_kind_it_is() {
    let page = drawn(&[
        Piece::title("TCP", "0.0.0.0:4444"),
        Piece::Blank,
        Piece::field("user", "www-data"),
        Piece::heading("WHAT IS HOLDING IT"),
        Piece::text("a quiet sentence"),
        Piece::warning("a loud one"),
    ]);

    assert!(page.contains("TCP"), "{page}");
    assert!(page.contains("0.0.0.0:4444"), "{page}");
    assert!(page.contains("user"), "{page}");
    assert!(page.contains("www-data"), "{page}");
    assert!(page.contains("WHAT IS HOLDING IT"), "{page}");
    assert!(page.contains("a quiet sentence"), "{page}");
    assert!(page.contains("a loud one"), "{page}");
}

#[test]
fn the_key_a_module_hands_over_is_drawn_as_something_an_operator_can_copy() {
    let page = drawn(&[Piece::key("port.listen|tcp|0.0.0.0:4444")]);

    assert!(page.contains("SUPPRESS"), "{page}");
    assert!(page.contains("suppressions:"), "{page}");
    assert!(page.contains("port.listen|tcp|0.0.0.0:4444"), "{page}");
}

#[test]
fn nothing_selected_is_a_sentence_rather_than_an_empty_panel() {
    let page = drawn(&[]);

    assert!(page.contains("Nothing is selected."), "{page}");
    assert_eq!(height(&[], fixture::look(), 80), 0);
}
