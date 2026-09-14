use vigil_model::Snapshot;

use super::listing::the_index_lists_every_search_and_sort_as_the_rows_do_in;
use super::tallying::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in;
use crate::{Facet, Pane, Showing};

const ROWS_ASKED_FOR_FACETS: usize = 3;

const HEADINGS_OPENED_ONE_BY_ONE: usize = 3;

const NO_HEADING: &str = "a heading no row of this list is called";

pub fn every_view_answers_from_its_index_and_its_counts_as_from_the_reading(
    pane: &dyn Pane,
    reading: &Snapshot,
) {
    every_view(pane, reading, &mut |view| {
        the_index_lists_every_search_and_sort_as_the_rows_do_in(pane, reading, view);
        the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(pane, reading, view);
    });
}

pub fn every_view(pane: &dyn Pane, reading: &Snapshot, check: &mut dyn FnMut(Showing<'_>)) {
    let toggles: Vec<&str> = pane.toggles().iter().map(|toggle| toggle.name).collect();
    let mut hidings: Vec<Vec<&str>> = toggles.iter().map(|name| vec![*name]).collect();
    if toggles.len() > 1 {
        hidings.push(toggles.clone());
    }

    let mut arrangements: Vec<Option<&str>> = vec![None];
    arrangements.extend(pane.arrangements().iter().map(|one| Some(one.name)));

    let facets = facets(pane, reading);

    check(Showing::default());
    for hidden in &hidings {
        check(Showing::default().hiding(hidden));
    }
    for only in &facets {
        check(Showing::default().narrowing(only));
    }
    for arranged in &arrangements {
        let around = Showing {
            arranged: *arranged,
            ..Showing::default()
        };
        for opened in openings(pane, reading, around) {
            let names: Vec<&str> = opened.iter().map(String::as_str).collect();
            check(Showing {
                opened: &names,
                ..around
            });
        }
    }

    let everything: Vec<&str> = toggles;
    let arranged = arrangements.last().copied().flatten();
    let only: &[Facet] = facets.first().map_or(&[], Vec::as_slice);
    let around = Showing {
        arranged,
        ..Showing::default()
    }
    .hiding(&everything)
    .narrowing(only);
    for opened in openings(pane, reading, around) {
        let names: Vec<&str> = opened.iter().map(String::as_str).collect();
        check(Showing {
            opened: &names,
            ..around
        });
    }
}

fn facets(pane: &dyn Pane, reading: &Snapshot) -> Vec<Vec<Facet>> {
    let mut seen: Vec<Facet> = Vec::new();
    for row in pane
        .rows(reading, &Showing::default())
        .iter()
        .take(ROWS_ASKED_FOR_FACETS)
    {
        for facet in pane.facets(reading, row) {
            if !seen.contains(&facet) {
                seen.push(facet);
            }
        }
    }

    let mut facets: Vec<Vec<Facet>> = seen.iter().map(|facet| vec![facet.clone()]).collect();
    let mut names: Vec<&'static str> = seen.iter().map(|facet| facet.name).collect();
    names.sort_unstable();
    names.dedup();
    for name in &names {
        facets.push(vec![Facet::new(name, "\u{2603} nothing is called this")]);
    }
    let mut one_of_each: Vec<Facet> = Vec::new();
    for facet in &seen {
        if !one_of_each.iter().any(|chosen| chosen.name == facet.name) {
            one_of_each.push(facet.clone());
        }
    }
    if one_of_each.len() > 1 {
        facets.push(one_of_each);
    }
    facets
}

fn openings(pane: &dyn Pane, reading: &Snapshot, around: Showing<'_>) -> Vec<Vec<String>> {
    let headings: Vec<String> = pane
        .rows(reading, &around)
        .into_iter()
        .filter(|row| row.opens())
        .map(|row| row.key)
        .collect();

    let mut openings: Vec<Vec<String>> = vec![Vec::new()];
    if headings.is_empty() {
        return openings;
    }
    openings.extend(
        headings
            .iter()
            .take(HEADINGS_OPENED_ONE_BY_ONE)
            .map(|heading| vec![heading.clone()]),
    );
    openings.push(headings);
    openings.push(vec![NO_HEADING.to_string()]);
    openings
}
