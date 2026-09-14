use vigil_model::Snapshot;

use crate::{Pane, Room, Showing, Sorting, listed};

const ROOMS: [u16; 2] = [80, 160];

pub fn the_index_lists_every_search_and_sort_as_the_rows_do(pane: &dyn Pane, reading: &Snapshot) {
    let Some(index) = pane.index(reading, &Showing::default()) else {
        return;
    };

    let mut searches: Vec<String> = vec![String::new(), "\u{2603}\u{2603} nothing".to_string()];
    for at in (0..index.len()).step_by((index.len() / 4).max(1)).take(5) {
        let text: Vec<char> = index.haystack(at).chars().collect();
        for (from, length) in [(0, 1), (0, 3), (text.len() / 2, 4)] {
            let piece: String = text.iter().skip(from).take(length).collect();
            if !piece.trim().is_empty() {
                searches.push(piece.to_uppercase());
                searches.push(piece);
            }
        }
    }

    let mut sortings = vec![Sorting::default()];
    if pane.offers().sorting {
        for by in 1..=pane.sorted_by().len() {
            for descending in [false, true] {
                sortings.push(Sorting { by, descending });
            }
        }
    }

    for search in &searches {
        for sorting in &sortings {
            let showing = Showing::searching(search).sorted(*sorting);
            let (found, from_the_index) = listed(pane, reading, &showing, &index, None);
            assert_eq!(
                from_the_index,
                pane.rows(reading, &showing),
                "{} lists different rows from its index than from the reading for the search \
                 {search:?} sorted {sorting:?}: the console answers every keystroke from the \
                 index, so a difference here is a list that changes when nothing in the host \
                 did",
                pane.name()
            );

            let longer = format!("{search}e");
            let narrowed = Showing::searching(&longer).sorted(*sorting);
            assert_eq!(
                listed(pane, reading, &narrowed, &index, Some(&found)).1,
                pane.rows(reading, &narrowed),
                "{} narrows the search {search:?} to {longer:?} from what the shorter search \
                 found and lists something the reading does not",
                pane.name()
            );
        }
    }
}

pub fn the_tally_from_the_counts_says_what_the_tally_from_the_reading_says(
    pane: &dyn Pane,
    reading: &Snapshot,
) {
    let Some(counts) = pane.counts(reading, &Showing::default()) else {
        return;
    };

    let mut searches: Vec<String> = vec![String::new(), "\u{2603}\u{2603} nothing".to_string()];
    for row in pane.rows(reading, &Showing::default()).iter().take(4) {
        let text: Vec<char> = row.key.chars().collect();
        searches.push(text.iter().take(2).collect());
        searches.push(text.iter().skip(text.len() / 2).take(3).collect());
    }

    for search in &searches {
        for note in [None, Some("part of the reading was refused")] {
            for elsewhere in [0, 2] {
                let showing = Showing {
                    elsewhere,
                    ..Showing::searching(search).noting(note)
                };
                let rows = pane.rows(reading, &showing);
                assert_eq!(
                    pane.tally_listed(reading, &showing, &rows, &counts),
                    pane.tally(reading, &showing, rows.len()),
                    "{} writes a different footer from what it counted once than from the whole reading \
                     for the search {search:?}: the console writes the footer from the counts on every \
                     keystroke",
                    pane.name()
                );
            }
        }
    }
}

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
    the_index_lists_every_search_and_sort_as_the_rows_do(pane, reading);
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says(pane, reading);

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
