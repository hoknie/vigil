use std::cmp::Reverse;

use vigil_model::Snapshot;

use crate::{Index, Pane, Showing, Sorting, listed};

const HAYSTACKS_TAKEN_APART: usize = 6;

const LONGEST_TYPED: usize = 6;

const SEARCHES_SORTED_APART: usize = 7;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz0123456789|.:/-% ";

const TYPED_NEXT: [&str; 3] = ["e", ".1", "|"];

pub fn the_index_lists_every_search_and_sort_as_the_rows_do(pane: &dyn Pane, reading: &Snapshot) {
    the_index_lists_every_search_and_sort_as_the_rows_do_in(pane, reading, Showing::default());
}

pub fn the_index_lists_every_search_and_sort_as_the_rows_do_in(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
) {
    the_index_lists_every_search_and_sort_as_the_rows_do_also(pane, reading, around, &[]);
}

pub fn the_index_lists_every_search_and_sort_as_the_rows_do_also(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
    also: &[&str],
) {
    let bare = Showing {
        search: "",
        sorting: Sorting::default(),
        ..around
    };
    let Some(index) = pane.index(reading, &bare) else {
        return;
    };

    let searched = searches(&index, also);
    let sorted_on = sorted_on(&searched);
    for sorting in sortings(pane) {
        let everything = sorting == Sorting::default();
        let mut found_by: Vec<(String, Vec<usize>)> = Vec::new();
        for search in searched
            .iter()
            .filter(|search| everything || sorted_on.contains(search))
        {
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
            found_by.push((search.clone(), found));
        }
        if everything {
            narrowed(pane, reading, around, sorting, &index, &found_by);
            typed(pane, reading, around, sorting, &index);
        }
    }
}

fn sorted_on(searched: &[String]) -> Vec<String> {
    let mut sorted_on: Vec<String> = ["", "e", "qqqzzz nothing of the sort"]
        .map(String::from)
        .to_vec();
    sorted_on.extend(
        searched
            .iter()
            .filter(|search| search.chars().count() > 2)
            .step_by(SEARCHES_SORTED_APART)
            .cloned(),
    );
    sorted_on
}

fn narrowed(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
    sorting: Sorting,
    index: &Index,
    found_by: &[(String, Vec<usize>)],
) {
    for (search, found) in found_by {
        for next in TYPED_NEXT {
            let longer = format!("{search}{next}");
            let showing = Showing {
                search: &longer,
                sorting,
                ..around
            };
            assert_eq!(
                listed(pane, reading, &showing, index, Some(found)).1,
                pane.rows(reading, &showing),
                "{} narrows {search:?} to {longer:?} sorted {sorting:?} around {around:?} from \
                 what the search before the keystroke found and lists what the reading does not",
                pane.name()
            );
        }
        let longer = search.to_lowercase();
        let mut shorter: Vec<&(String, Vec<usize>)> = found_by
            .iter()
            .filter(|(before, _)| {
                let before = before.to_lowercase();
                !before.is_empty() && before != longer && longer.contains(before.as_str())
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
                listed(pane, reading, &showing, index, Some(found)).1,
                pane.rows(reading, &showing),
                "{} narrows {before:?} to {search:?} sorted {sorting:?} around {around:?} from \
                 what the shorter search found and lists what the reading does not: the console \
                 reuses the last answer while the reader keeps typing",
                pane.name()
            );
        }
    }
}

fn typed(
    pane: &dyn Pane,
    reading: &Snapshot,
    around: Showing<'_>,
    sorting: Sorting,
    index: &Index,
) {
    for chain in typings(index) {
        let mut within: Option<Vec<usize>> = None;
        let mut before: Option<&String> = None;
        for search in &chain {
            let showing = Showing {
                search,
                sorting,
                ..around
            };
            let (found, rows) = listed(pane, reading, &showing, index, within.as_deref());
            assert_eq!(
                rows,
                pane.rows(reading, &showing),
                "{} types {search:?} after {before:?} sorted {sorting:?} around {around:?} and \
                 lists other rows from the index than from the reading",
                pane.name()
            );
            within = Some(found);
            before = Some(search);
        }
    }
}

pub(super) fn searches(index: &Index, also: &[&str]) -> Vec<String> {
    let mut searches: Vec<String> = [
        "",
        "\u{2603}",
        "qqqzzz nothing of the sort",
        "A",
        "S",
        "Root",
    ]
    .map(String::from)
    .to_vec();
    searches.extend(also.iter().map(|search| search.to_string()));
    searches.extend(LETTERS.chars().map(String::from));
    for text in taken_apart(index) {
        let pieces = [
            (0, 2),
            (0, LONGEST_TYPED),
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

fn typings(index: &Index) -> Vec<Vec<String>> {
    let mut typings = vec![
        vec![String::new(), "e".to_string(), "e\u{2603}".to_string()],
        vec![String::new(), "S".to_string(), "SE".to_string()],
    ];
    for text in taken_apart(index) {
        for from in [0, text.len() / 2, text.len().saturating_sub(LONGEST_TYPED)] {
            let chain: Vec<String> = (1..=LONGEST_TYPED)
                .map(|length| text.iter().skip(from).take(length).collect::<String>())
                .collect();
            let shouted: Vec<String> = chain.iter().map(|piece| piece.to_uppercase()).collect();
            if let Some(last) = chain.last().cloned() {
                typings.push(vec![last.clone(), format!("{last}\u{2603}")]);
            }
            typings.push(chain);
            typings.push(shouted);
        }
    }
    typings
}

fn taken_apart(index: &Index) -> Vec<Vec<char>> {
    let step = (index.len() / HAYSTACKS_TAKEN_APART).max(1);
    (0..index.len())
        .step_by(step)
        .map(|at| index.haystack(at).chars().collect())
        .collect()
}

pub(super) fn sortings(pane: &dyn Pane) -> Vec<Sorting> {
    let mut sortings = vec![Sorting::default()];
    for by in 1..=pane.sorted_by().len() + 1 {
        for descending in [false, true] {
            sortings.push(Sorting { by, descending });
        }
    }
    sortings
}
