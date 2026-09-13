use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Showing, render};
use crate::ui::helpers::words::text;
use crate::ui::{Arrows, Search, Sorting, fixture, holding};

fn drawn(at: usize, search: &Search, hidden: &[String], width: u16) -> String {
    let view = fixture::view();
    let section = holding("ports").expect("this build draws the sockets");
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 24));

    render(
        &view,
        fixture::look(),
        section.as_ref(),
        &Showing {
            at,
            search,
            hidden,
            cursor: 0,
            arrows: Arrows::List,
            sorting: Sorting::default(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
        },
        buffer.area,
        &mut buffer,
    );

    text::to_text(&buffer)
}

#[test]
fn the_table_a_pane_describes_is_the_table_that_is_drawn() {
    let page = drawn(0, &Search::default(), &[], 120);

    assert!(page.contains("PROTO"), "{page}");
    assert!(page.contains("ADDRESS"), "{page}");
    assert!(page.contains("0.0.0.0:22"), "{page}");
    assert!(page.contains("sshd"), "{page}");
}

#[test]
fn a_section_with_more_than_one_pane_draws_the_row_of_names_over_it() {
    let page = drawn(0, &Search::default(), &[], 120);

    assert!(page.contains("sockets"), "{page}");
    assert!(page.contains("by program"), "{page}");
}

#[test]
fn the_second_pane_of_a_section_is_the_one_that_is_drawn_when_it_is_chosen() {
    let grouped = drawn(1, &Search::default(), &[], 120);

    assert!(grouped.contains("PROGRAM"), "{grouped}");
    assert!(grouped.contains("grouped by program"), "{grouped}");
}

#[test]
fn a_kind_the_reader_switched_off_leaves_the_table_and_is_named_in_the_footer() {
    let hidden = vec!["tcp".to_string()];

    let page = drawn(0, &Search::default(), &hidden, 120);

    assert!(!page.contains("0.0.0.0:4444"), "{page}");
    assert!(page.contains("hiding tcp"), "{page}");
}

#[test]
fn a_pane_with_nothing_left_to_show_says_so_instead_of_drawing_an_empty_table() {
    let hidden = vec![
        "tcp".to_string(),
        "tcp6".to_string(),
        "udp".to_string(),
        "udp6".to_string(),
        "unix".to_string(),
    ];

    let page = drawn(0, &Search::default(), &hidden, 120);

    assert!(page.contains("No socket here is"), "{page}");
}

#[test]
fn a_narrow_terminal_drops_the_column_the_pane_said_needs_room() {
    let narrow = drawn(0, &Search::default(), &[], 80);
    let wide = drawn(0, &Search::default(), &[], 160);

    assert!(!narrow.contains("COMMAND"), "{narrow}");
    assert!(wide.contains("COMMAND"), "{wide}");
}
