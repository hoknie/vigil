use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_view::Piece;

use super::{height, render};
use crate::ui::fixture;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::words::text;

fn drawn(pieces: &[Piece]) -> String {
    acting(pieces, Acts::of_a_row(None))
}

fn acting(pieces: &[Piece], acts: Acts) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
    render(
        pieces,
        acts,
        None,
        fixture::look(),
        0,
        buffer.area,
        &mut buffer,
    );
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
fn the_panel_of_a_row_draws_a_button_for_everything_that_can_be_done_to_it() {
    let acts_on_it = acting(
        &[Piece::key("port.listen|tcp|0.0.0.0:4444")],
        Acts::of_a_row(Some(vigil_model::KillTarget::Socket)),
    );

    assert!(acts_on_it.contains("ACTIONS"), "{acts_on_it}");
    for button in ["[ K close this socket ]", "[ S suppress it ]"] {
        assert!(
            acts_on_it.contains(button),
            "{button} is missing: {acts_on_it}"
        );
    }
}

#[test]
fn a_row_nothing_can_be_done_to_draws_no_buttons_and_no_heading_over_them() {
    let page = drawn(&[Piece::key("port.listen|tcp|0.0.0.0:4444")]);

    assert!(page.contains("SUPPRESS"), "{page}");
    assert!(
        !page.contains("ACTIONS"),
        "a heading with nothing under it is a heading that asks a reader to look for \
         something that is not there: {page}"
    );
}

#[test]
fn nothing_selected_is_a_sentence_rather_than_an_empty_panel() {
    let page = drawn(&[]);

    assert!(page.contains("Nothing is selected."), "{page}");
    assert_eq!(
        height(
            &[],
            Acts::of_a_row(Some(vigil_model::KillTarget::Socket)),
            fixture::look(),
            80
        ),
        0
    );
}

#[test]
fn a_drawn_line_reaches_the_screen_as_it_was_written_and_is_never_wrapped_or_indented() {
    let path = "in ──▶ prerouting ──▶ ( routing ) ──▶ input ──▶ this host, and a tail long \
                enough that prose of this length would be broken in two";

    let page = drawn(&[Piece::line(format!("   {path}"))]);
    let first = page.lines().next().expect("a page has a first line");

    assert!(
        first.starts_with("   in ──▶ prerouting"),
        "a drawn line carries its own indent, and three columns added to it move every arrow \
         of a diagram away from what it points at: {first:?}"
    );
    assert!(
        !page
            .lines()
            .any(|line| line.trim_start().starts_with("enough that prose")),
        "the remainder of a wrapped line lands under the first column and turns a diagram \
         into a column of fragments: {page}"
    );
}
