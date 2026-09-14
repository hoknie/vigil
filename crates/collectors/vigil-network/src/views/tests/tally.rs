use vigil_view::conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in;
use vigil_view::{Facet, Pane, Section, Showing};

use super::super::Listening;
use super::index::{KINDS_SWITCHED_OFF, scaled};
use crate::fixture::ports;

fn panes() -> Vec<Box<dyn Pane>> {
    Listening.panes()
}

#[test]
fn the_footer_of_either_list_with_kinds_switched_off_says_what_the_whole_reading_says() {
    for pane in panes() {
        for reading in [ports(), scaled(4)] {
            the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
                pane.as_ref(),
                &reading,
                Showing::default(),
            );
            for hidden in KINDS_SWITCHED_OFF {
                the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
                    pane.as_ref(),
                    &reading,
                    Showing::default().hiding(hidden),
                );
            }
        }
    }
}

#[test]
fn the_footer_of_the_tree_opened_at_its_headings_counts_the_rows_it_lists_and_not_the_sockets() {
    let pane = &panes()[1];
    let reading = scaled(3);
    let headings: Vec<String> = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .map(|row| row.key)
        .collect();
    let every: Vec<&str> = headings.iter().map(String::as_str).collect();

    for opened in [&every[..1], &["unresolved"][..], &every[..]] {
        for hidden in [&[][..], KINDS_SWITCHED_OFF[0]] {
            the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
                pane.as_ref(),
                &reading,
                Showing::default().opening(opened).hiding(hidden),
            );
        }
    }
}

#[test]
fn a_facet_or_an_arrangement_the_console_keeps_writes_the_same_footer_either_way() {
    let facets = [Facet::new("user", "root")];

    for pane in panes() {
        for around in [
            Showing::default().narrowing(&facets),
            Showing::default().arranged("by program"),
            Showing::default()
                .narrowing(&facets)
                .hiding(KINDS_SWITCHED_OFF[1]),
        ] {
            the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
                pane.as_ref(),
                &ports(),
                around,
            );
        }
    }
}
