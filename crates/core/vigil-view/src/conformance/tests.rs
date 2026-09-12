use serde_json::json;
use vigil_model::Snapshot;

use super::suite;
use crate::{Cell, Column, Notice, Pane, Room, RowKey, Showing, Width};

struct Listening {
    short: bool,
}

impl Pane for Listening {
    fn name(&self) -> &'static str {
        "flat"
    }

    fn about(&self) -> &'static str {
        "every socket this host listens on"
    }

    fn reads(&self) -> &'static str {
        "ports"
    }

    fn columns(&self, _room: Room) -> Vec<Column> {
        vec![
            Column::new("PROTO", Width::Fixed(5)),
            Column::new("ADDRESS", Width::Least(22)),
        ]
    }

    fn rows(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Vec<RowKey> {
        reading.items.keys().map(RowKey::of).collect()
    }

    fn cells(&self, _reading: &Snapshot, row: &RowKey, _room: Room) -> Vec<Cell> {
        match self.short {
            true => vec![Cell::plain(row.key.clone())],
            false => vec![Cell::plain("tcp"), Cell::plain(row.key.clone())],
        }
    }

    fn detail(&self, _reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<crate::Piece> {
        vec![crate::Piece::field("key", row.key.clone())]
    }

    fn tally(&self, reading: &Snapshot, _shown: usize) -> String {
        format!("{} sockets", reading.items.len())
    }

    fn empty(&self, _showing: &Showing<'_>) -> Notice {
        Notice::plain("nothing is listening")
    }
}

fn reading() -> Snapshot {
    Snapshot::new("ports", "2026-09-12T10:00:00.000Z".to_string()).with(
        "tcp|0.0.0.0:443",
        json!({ "protocol": "tcp", "address": "0.0.0.0", "port": 443 }),
    )
}

#[test]
fn a_pane_that_answers_about_its_own_reading_passes_the_whole_suite() {
    suite::run_all(&Listening { short: false }, &reading());
}

#[test]
#[should_panic(expected = "cells under")]
fn a_row_shorter_than_the_headers_is_caught_here_and_not_by_a_reader_of_a_shifted_table() {
    suite::a_pane_draws_a_cell_for_every_column_it_declares(&Listening { short: true }, &reading());
}

#[test]
#[should_panic(expected = "was handed the reading of")]
fn a_pane_given_somebody_elses_reading_is_caught_before_it_draws_it() {
    let elsewhere = Snapshot::new("users", "2026-09-12T10:00:00.000Z".to_string());

    suite::a_pane_reads_the_snapshot_it_says_it_reads(&Listening { short: false }, &elsewhere);
}
