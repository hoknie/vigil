use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Showing, render};
use crate::ui::helpers::words::text;
use crate::ui::{Arrows, Search, Sorting, fixture, holding};

fn drawn(at: usize, search: &Search, hidden: &[String], width: u16) -> String {
    drawn_with(at, search, hidden, width, Vec::new(), Vec::new())
}

fn drawn_with(
    at: usize,
    search: &Search,
    hidden: &[String],
    width: u16,
    marked: Vec<String>,
    opened: Vec<String>,
) -> String {
    let view = fixture::view();
    let section = holding("network").expect("this build draws the sockets");
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
            group: None,
            sorting: Sorting::default(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
            marked,
            opened,
            only: &[],
            listed: None,
            tally: None,
        },
        buffer.area,
        &mut buffer,
    );

    text::to_text(&buffer)
}

#[test]
fn rows_and_a_tally_the_console_already_holds_are_drawn_as_given_and_not_worked_out_again() {
    let view = fixture::view();
    let section = holding("network").expect("this build draws the sockets");
    let search = Search::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 120, 24));

    render(
        &view,
        fixture::look(),
        section.as_ref(),
        &Showing {
            at: 0,
            search: &search,
            hidden: &[],
            cursor: 0,
            arrows: Arrows::List,
            group: None,
            sorting: Sorting::default(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
            marked: Vec::new(),
            opened: Vec::new(),
            only: &[],
            listed: Some(std::rc::Rc::new(crate::ui::types::cache::Shown::built(
                Vec::new(),
            ))),
            tally: Some("the tally the console kept".to_string()),
        },
        buffer.area,
        &mut buffer,
    );
    let page = text::to_text(&buffer);

    assert!(
        !page.contains("0.0.0.0:22"),
        "the reading holds this socket, and the list was told it holds no rows: a list that \
         asks the pane again on every keypress is the work the console keeps so it does not \
         have to: {page}"
    );
    assert!(page.contains("the tally the console kept"), "{page}");
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

#[test]
fn a_socket_the_reader_marked_carries_a_character_beside_it_and_not_only_a_colour() {
    let page = drawn_with(
        0,
        &Search::default(),
        &[],
        120,
        vec!["tcp|0.0.0.0:22".to_string()],
        Vec::new(),
    );

    let row = page
        .lines()
        .find(|line| line.contains("0.0.0.0:22"))
        .expect("the sample listens on 22");

    assert!(
        row.contains('x'),
        "what a kill is aimed at has to be visible on a terminal with no colour at all: {page}"
    );
    assert!(
        !page
            .lines()
            .find(|line| line.contains("0.0.0.0:4444"))
            .is_some_and(|other| other.starts_with(" x") || other.starts_with(" > x")),
        "and only what was marked: {page}"
    );
}

#[test]
fn the_tree_draws_programs_alone_until_one_of_them_is_opened() {
    let closed = drawn_with(1, &Search::default(), &[], 120, Vec::new(), Vec::new());

    assert!(closed.contains("nginx"), "{closed}");
    assert!(
        !closed.contains("0.0.0.0:443"),
        "a tree nobody opened must not be drawing leaves: {closed}"
    );
    assert!(
        closed.contains('\u{25b8}'),
        "a closed branch says so: {closed}"
    );

    let opened = drawn_with(
        1,
        &Search::default(),
        &[],
        120,
        Vec::new(),
        vec!["program|/usr/sbin/nginx".to_string()],
    );

    assert!(opened.contains(":::443"), "{opened}");
    assert!(
        opened.contains('\u{25be}'),
        "an open branch says so: {opened}"
    );
}

fn said(line: &ratatui::text::Line<'static>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn a_row_of_names_wider_than_the_terminal_folds_to_the_one_name_and_its_neighbours() {
    let named: Vec<(usize, String)> = ["containers", "images", "volumes", "networks", "compose"]
        .iter()
        .enumerate()
        .map(|(at, name)| (at, (*name).to_string()))
        .collect();

    let (whole, places) = super::menu::row(fixture::look(), &named, 1, true, 80);
    assert_eq!(
        places.len(),
        named.len(),
        "with room for every name, every name is on the row and every one of them can be \
         clicked: {}",
        said(&whole)
    );

    let (folded, places) = super::menu::row(fixture::look(), &named, 1, true, 20);
    let drawn = said(&folded);

    assert!(
        drawn.contains("< [images] >"),
        "a row that does not fit is folded onto the name the reader is on, with the way to \
         the next on either side of it, rather than being cut off at the frame: {drawn}"
    );
    assert_eq!(
        places.iter().map(|(at, ..)| *at).collect::<Vec<usize>>(),
        vec![0, 1, 2],
        "and the arrows are the neighbours, so a click on one is a step along the row"
    );
    assert!(
        drawn.chars().count() <= 20,
        "a folded row fits the terminal it was folded for: {drawn}"
    );
}

#[test]
fn the_row_the_arrows_are_on_carries_the_caret_and_the_other_row_does_not() {
    let named = vec![(0, "host".to_string()), (1, "docker".to_string())];

    let here = said(&super::menu::row(fixture::look(), &named, 0, true, 80).0);
    let away = said(&super::menu::row(fixture::look(), &named, 0, false, 80).0);

    assert!(here.starts_with(" \u{25b8} "), "{here}");
    assert!(
        away.starts_with("   ") && !away.contains('\u{25b8}'),
        "which row the arrows walk is readable with no colour at all: {away}"
    );
    assert!(
        away.contains("[host]"),
        "and the chosen name is still marked on the row that is not focused: {away}"
    );
}
