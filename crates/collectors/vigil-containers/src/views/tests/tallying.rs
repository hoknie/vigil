use vigil_model::Snapshot;
use vigil_view::{Pane, Showing, Sorting};

use super::listing::searches;

pub(super) fn tallies_as_the_reading_does(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
) {
    let bare = Showing {
        search: "",
        sorting: Sorting::default(),
        ..around
    };
    let index = pane
        .index(reading, &bare)
        .expect("every pane of this crate answers from an index");

    for note in [None, Some("part of the reading was refused")] {
        for elsewhere in [0, 3] {
            let around = Showing {
                note,
                elsewhere,
                ..around
            };
            let bare = Showing {
                search: "",
                sorting: Sorting::default(),
                ..around
            };
            let counts = pane
                .counts(reading, &bare)
                .expect("every pane of this crate counts its reading once");

            for search in &searches(&index) {
                for sorting in [
                    Sorting::default(),
                    Sorting {
                        by: 1,
                        descending: true,
                    },
                ] {
                    let showing = Showing {
                        search,
                        sorting,
                        ..around
                    };
                    let rows = pane.rows(reading, &showing);
                    assert_eq!(
                        pane.tally_listed(reading, &showing, &rows, &counts),
                        pane.tally(reading, &showing, rows.len()),
                        "{} writes a different footer from what it counted once than from the \
                         whole reading for the search {search:?} around {showing:?}: the console \
                         writes the footer from the counts on every keystroke",
                        pane.name()
                    );
                }
            }
        }
    }
}
