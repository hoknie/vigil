use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_view::{RowKey, Section};

use super::render;
use super::render::{CAPTION, counting};
use crate::ui::helpers::words::text;
use crate::ui::{Graph, Motion, fixture};

const ETH0: &str = "fw-interface|eth0";

const INTERFACES: usize = 2;

fn opened() -> Graph {
    let reading = vigil_firewall::fixture::firewall();
    let pane = vigil_firewall::WhatTheHostLetsIn.panes().remove(INTERFACES);
    let row = RowKey::of(ETH0);

    Graph::open(row.clone(), pane.graph(&reading, &row))
}

fn drawn(graph: &Graph, width: u16, height: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    let pieces = graph.pieces(std::time::Instant::now(), Some(520_614));
    render(
        &pieces,
        graph.watching(),
        fixture::look(),
        graph.top(),
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn the_panel_puts_the_way_back_and_the_counting_key_on_top_of_the_drawing() {
    let page = drawn(&opened(), 80, 40);
    let lines: Vec<&str> = page.lines().collect();

    assert!(lines[0].contains("[ \u{2190} Back ]"), "{page}");
    assert!(lines[0].contains("start counting"), "{page}");
    assert!(lines[0].contains(CAPTION), "{page}");
}

#[test]
fn the_drawing_names_the_interface_the_hooks_and_the_way_a_packet_leaves() {
    let page = drawn(&opened(), 80, 40);

    assert!(page.contains("eth0"), "{page}");
    assert!(page.contains("prerouting"), "{page}");
    assert!(page.contains("this host"), "{page}");
}

#[test]
fn the_panel_fits_eighty_columns_with_no_colour_and_never_runs_off_the_side() {
    for width in [80u16, 120] {
        let page = drawn(&opened(), width, 60);
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width}: {line:?} — the path is drawn as it is and never rewrapped, so a \
                 line wider than the terminal is a line cut in half"
            );
        }
    }
}

#[test]
fn the_key_on_top_says_what_pressing_it_will_do_rather_than_what_it_did() {
    assert!(counting(false).contains("start counting"));
    assert!(counting(true).contains("stop counting"));
    assert!(
        counting(false).contains('w') && counting(true).contains('w'),
        "the label names the key, so the two cannot drift apart"
    );
}

#[test]
fn a_panel_that_is_counting_shows_what_went_through_and_one_that_is_not_says_how_to_start() {
    let quiet = drawn(&opened(), 80, 60);
    assert!(quiet.contains("Counting is off"), "{quiet}");

    let mut watched = opened();
    watched.watch(std::time::Instant::now());
    watched.note(Some(520_614));
    let page = drawn(&watched, 80, 60);

    assert!(page.contains("520614 packet(s)"), "{page}");
    assert!(page.contains("stop counting"), "{page}");
}

#[test]
fn a_panel_scrolled_down_shows_the_later_lines_and_keeps_the_way_back_on_top() {
    let mut graph = opened();
    graph.scroll(Motion::Last, 8, 60);

    let page = drawn(&graph, 80, 12);

    assert!(page.contains("[ \u{2190} Back ]"), "{page}");
    assert!(
        !page.contains("arriving on eth0"),
        "a panel scrolled to the end that still shows its first line has not scrolled: {page}"
    );
}
