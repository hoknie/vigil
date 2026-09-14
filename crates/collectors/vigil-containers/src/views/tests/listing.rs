use std::cmp::Reverse;

use vigil_model::Snapshot;
use vigil_view::{Index, Pane, Showing, Sorting, listed};

const HAYSTACKS_TAKEN_APART: usize = 6;

pub(super) fn searches(index: &Index) -> Vec<String> {
    let mut searches: Vec<String> = ["", "\u{2603}", "qqqzzz", "A", "S", "E", "Root"]
        .map(String::from)
        .to_vec();
    searches.extend("aeinorstu0123456789|.:/- ".chars().map(String::from));
    let step = (index.len() / HAYSTACKS_TAKEN_APART).max(1);
    for at in (0..index.len()).step_by(step) {
        let text: Vec<char> = index.haystack(at).chars().collect();
        let pieces = [
            (0, 2),
            (0, 6),
            (text.len() / 3, 3),
            (text.len() / 2, 8),
            (text.len().saturating_sub(5), 5),
        ];
        for (from, length) in pieces {
            let piece: String = text.iter().skip(from).take(length).collect();
            if !piece.is_empty() {
                searches.push(piece.to_uppercase());
                searches.push(piece);
            }
        }
    }
    searches.sort();
    searches.dedup();
    searches
}

fn sortings(pane: &dyn Pane) -> Vec<Sorting> {
    let mut sortings = vec![Sorting::default()];
    for by in 1..=pane.sorted_by().len() + 1 {
        for descending in [false, true] {
            sortings.push(Sorting { by, descending });
        }
    }
    sortings
}

pub(super) fn lists_as_the_rows_do(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
) -> usize {
    let bare = Showing {
        search: "",
        sorting: Sorting::default(),
        ..around
    };
    let index = pane
        .index(reading, &bare)
        .expect("every pane of this crate answers from an index");
    let searches = searches(&index);

    for sorting in sortings(pane) {
        let mut found_by: Vec<(String, Vec<usize>)> = Vec::new();
        for search in &searches {
            let showing = Showing {
                search,
                sorting,
                ..around
            };
            let (found, rows) = listed(pane, reading, &showing, &index, None);
            assert_eq!(
                rows,
                pane.rows(reading, &showing),
                "{} lists from its index something other than its rows for the search \
                 {search:?} sorted {sorting:?} around {around:?}: the console answers every \
                 keystroke from the index, and a list that differs is one that changes when \
                 nothing on the host did",
                pane.name()
            );
            found_by.push((search.to_lowercase(), found));
        }

        for search in &searches {
            let longer = search.to_lowercase();
            let mut shorter: Vec<&(String, Vec<usize>)> = found_by
                .iter()
                .filter(|(before, _)| {
                    !before.is_empty() && *before != longer && longer.contains(before.as_str())
                })
                .collect();
            shorter.sort_by_key(|(before, _)| Reverse(before.chars().count()));
            for (before, found) in shorter.into_iter().take(2) {
                let showing = Showing {
                    search,
                    sorting,
                    ..around
                };
                assert_eq!(
                    listed(pane, reading, &showing, &index, Some(found)).1,
                    pane.rows(reading, &showing),
                    "{} narrows {before:?} to {search:?} sorted {sorting:?} around {around:?} \
                     from what the shorter search found and lists what the reading does not: \
                     the console reuses the last answer while the reader keeps typing",
                    pane.name()
                );
            }
        }
    }
    index.len()
}
