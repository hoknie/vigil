use vigil_model::Snapshot;

use crate::{Pane, Room, Showing};

const ROOMS: [u16; 2] = [80, 160];

pub fn a_pane_reads_the_snapshot_it_says_it_reads(pane: &dyn Pane, reading: &Snapshot) {
    assert_eq!(
        pane.reads(),
        reading.source,
        "the pane named {} was handed the reading of {}: the console picks a reading by this \
         name, and a pane that answers about another collector draws a screen nobody can \
         explain",
        pane.name(),
        reading.source
    );
}

pub fn a_pane_draws_a_cell_for_every_column_it_declares(pane: &dyn Pane, reading: &Snapshot) {
    for columns in ROOMS {
        let room = Room::of(columns);
        let declared = pane.columns(room).len();
        for row in pane.rows(reading, &Showing::default()) {
            let drawn = pane.cells(reading, &row, room).len();
            assert_eq!(
                drawn,
                declared,
                "{} draws {drawn} cells under {declared} columns at {columns} columns of \
                 terminal, on the row {}: the renderer lays cells against the headers it was \
                 given, so a row of a different length silently shifts every value one \
                 column to the left",
                pane.name(),
                row.key
            );
        }
    }
}

pub fn every_row_the_pane_offers_is_in_the_reading(pane: &dyn Pane, reading: &Snapshot) {
    for row in pane.rows(reading, &Showing::default()) {
        if !row.of_the_reading {
            continue;
        }
        assert!(
            reading.items.contains_key(&row.key),
            "{} offers the row {}, which the reading does not hold: a finding walks to its \
             object by this key, and a key of the pane's own making is a walk to nowhere",
            pane.name(),
            row.key
        );
    }
}

pub fn nothing_is_shown_twice_under_one_key(pane: &dyn Pane, reading: &Snapshot) {
    let rows = pane.rows(reading, &Showing::default());
    let mut keys: Vec<&str> = rows
        .iter()
        .filter(|row| row.of_the_reading)
        .map(|row| row.key.as_str())
        .collect();
    let total = keys.len();
    keys.sort_unstable();
    keys.dedup();

    assert_eq!(
        keys.len(),
        total,
        "{} offers the same key twice: the cursor is held by key, and two rows under one key \
         make it jump",
        pane.name()
    );
}

pub fn what_the_pane_shows_of_a_row_is_more_than_the_row_itself(
    pane: &dyn Pane,
    reading: &Snapshot,
) {
    if !pane.offers().detail {
        return;
    }
    for row in pane.rows(reading, &Showing::default()) {
        if !row.of_the_reading {
            continue;
        }
        assert!(
            !pane.detail(reading, &row, 80).is_empty(),
            "{} offers a detail and has nothing to say about {}: an empty pane under a row \
             reads as a console that lost the answer",
            pane.name(),
            row.key
        );
    }
}

pub fn a_pane_says_what_to_write_over_it_and_over_the_row_it_opens(pane: &dyn Pane) {
    assert!(
        !pane.caption().is_empty() && !pane.detail_caption().is_empty(),
        "{} draws a list and a detail with nothing written over either: the captions are \
         what tells a reader which of the two the keys are moving in",
        pane.name()
    );
}

pub fn a_pane_that_gathers_rows_starts_with_every_one_of_them_put_away(
    pane: &dyn Pane,
    reading: &Snapshot,
) {
    let closed = pane.rows(reading, &Showing::default());
    if !closed.iter().any(|row| row.opens()) {
        return;
    }

    assert!(
        closed.iter().all(|row| row.gathered_under.is_none()),
        "{} draws a row gathered under a heading before anybody opened that heading: a tree \
         that arrives open is a tree whose first screen is the flat list with extra lines in \
         it",
        pane.name()
    );

    for heading in closed.iter().filter(|row| row.opens()) {
        let open = [heading.key.as_str()];
        let rows = pane.rows(reading, &Showing::default().opening(&open));
        let under = rows
            .iter()
            .filter(|row| row.gathered_under.as_deref() == Some(heading.key.as_str()))
            .count();

        assert_eq!(
            under,
            heading.gathers.unwrap_or_default(),
            "{} writes {:?} on the heading {} and puts {under} row(s) under it when it is \
             opened: the number on a closed heading is the only thing a reader has to decide \
             whether opening it is worth the keystroke",
            pane.name(),
            heading.gathers,
            heading.key
        );
    }
}

pub fn run_all(pane: &dyn Pane, reading: &Snapshot) {
    a_pane_says_what_to_write_over_it_and_over_the_row_it_opens(pane);
    a_pane_reads_the_snapshot_it_says_it_reads(pane, reading);
    a_pane_draws_a_cell_for_every_column_it_declares(pane, reading);
    every_row_the_pane_offers_is_in_the_reading(pane, reading);
    nothing_is_shown_twice_under_one_key(pane, reading);
    what_the_pane_shows_of_a_row_is_more_than_the_row_itself(pane, reading);
    a_pane_that_gathers_rows_starts_with_every_one_of_them_put_away(pane, reading);

    assert!(
        !pane.tally(reading, &Showing::default(), 0).is_empty(),
        "{} draws a footer with nothing in it: the tally is where a reader learns how much of \
         the reading the screen is showing",
        pane.name()
    );
    assert!(
        !pane.empty(&Showing::default()).headline.is_empty(),
        "{} says nothing when it has nothing to show: an empty table is the moment a reader \
         needs a sentence most",
        pane.name()
    );
}
