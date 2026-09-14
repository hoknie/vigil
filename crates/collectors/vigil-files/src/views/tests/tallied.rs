use vigil_model::Snapshot;
use vigil_view::{Pane, Showing, Sorting};

const SAMPLES: usize = 6;

const NOTES: [Option<&str>; 2] = [None, Some("part of the reading was refused")];

const ELSEWHERE: [usize; 2] = [0, 2];

pub(crate) fn the_footer_from_the_counts_says_what_the_footer_from_the_reading_says(
    pane: &dyn Pane,
    reading: &Snapshot,
    showing: Showing<'_>,
) {
    let counts = pane
        .counts(reading, &showing)
        .expect("a pane that counts nothing once walks the whole reading for every footer");

    for search in searches(pane, reading, showing) {
        for sorting in sortings(pane) {
            for note in NOTES {
                for elsewhere in ELSEWHERE {
                    let now = Showing {
                        search: &search,
                        sorting,
                        note,
                        elsewhere,
                        ..showing
                    };
                    let rows = pane.rows(reading, &now);
                    assert_eq!(
                        pane.tally_listed(reading, &now, &rows, &counts),
                        pane.tally(reading, &now, rows.len()),
                        "{}: the footer for the search {search:?} sorted {sorting:?} says \
                         something else from the counts taken once than from the whole \
                         reading, and the console writes it from the counts on every keystroke",
                        pane.name()
                    );
                }
            }
        }
    }
}

fn searches(pane: &dyn Pane, reading: &Snapshot, showing: Showing<'_>) -> Vec<String> {
    let mut searches = vec![
        String::new(),
        "e".to_string(),
        "S".to_string(),
        "zzzz nothing of the sort".to_string(),
    ];
    let Some(index) = pane.index(reading, &showing) else {
        return searches;
    };
    let step = (index.len() / SAMPLES).max(1);
    for at in (0..index.len()).step_by(step) {
        let text: Vec<char> = index.haystack(at).chars().collect();
        for (from, length) in [(0, 1), (0, 4), (text.len() / 2, 3)] {
            let piece: String = text.iter().skip(from).take(length).collect();
            searches.push(piece.to_uppercase());
            searches.push(piece);
        }
    }
    searches
}

fn sortings(pane: &dyn Pane) -> Vec<Sorting> {
    let mut sortings = vec![Sorting::default()];
    if pane.offers().sorting {
        for by in 1..=pane.sorted_by().len() {
            sortings.push(Sorting {
                by,
                descending: true,
            });
        }
    }
    sortings
}
