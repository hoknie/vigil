use vigil_model::Snapshot;

use super::listing::searches;
use crate::{Pane, Rows, Showing, Sorting, listing};

const NOTES: [Option<&str>; 2] = [None, Some("part of the reading was refused")];

const ELSEWHERE: [usize; 2] = [0, 2];

pub fn the_tally_from_the_counts_says_what_the_tally_from_the_reading_says(
    pane: &dyn Pane,
    reading: &Snapshot,
) {
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
        pane,
        reading,
        Showing::default(),
    );
}

pub fn the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
) {
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_also(
        pane,
        reading,
        around,
        &[],
    );
}

pub fn the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_also(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
    also: &[&str],
) {
    let bare = Showing {
        search: "",
        sorting: Sorting::default(),
        only: &[],
        ..around
    };
    let Some(counts) = pane.counts(reading, &bare) else {
        return;
    };
    let index = pane.index(reading, &bare);
    let searched = match &index {
        Some(index) => searches(index, also),
        None => ["", "e", "S", "qqqzzz nothing of the sort"]
            .iter()
            .chain(also)
            .map(|search| search.to_string())
            .collect(),
    };

    for search in &searched {
        for sorting in sortings(pane) {
            for note in NOTES {
                let listed = Showing {
                    search,
                    sorting,
                    note,
                    ..around
                };
                let rows = pane.rows(reading, &listed);
                let assembled = index
                    .as_ref()
                    .map(|index| (index, listing(pane, reading, &listed, index, None).1));
                for elsewhere in ELSEWHERE {
                    let showing = Showing {
                        elsewhere,
                        ..listed
                    };
                    let from_the_reading = pane.tally(reading, &showing, rows.len());
                    assert_eq!(
                        pane.tally_listed(reading, &showing, &Rows::built(&rows), &counts),
                        from_the_reading,
                        "{} writes a different footer from what it counted once than from the \
                         whole reading for the search {search:?} sorted {sorting:?} around \
                         {around:?}: the console writes the footer from the counts on every \
                         keystroke",
                        pane.name()
                    );
                    if let Some((index, assembled)) = &assembled {
                        assert_eq!(
                            pane.tally_listed(
                                reading,
                                &showing,
                                &Rows::of(index, assembled),
                                &counts
                            ),
                            from_the_reading,
                            "{} writes a different footer from the row numbers its index listed \
                             than from the rows of the reading for the search {search:?} sorted \
                             {sorting:?} around {around:?}: the console keeps the row numbers \
                             and not copies of the rows",
                            pane.name()
                        );
                    }
                }
            }
        }
    }
}

fn sortings(pane: &dyn Pane) -> Vec<Sorting> {
    let mut sortings = vec![Sorting::default()];
    if !pane.sorted_by().is_empty() {
        sortings.push(Sorting {
            by: pane.sorted_by().len(),
            descending: true,
        });
    }
    sortings
}
