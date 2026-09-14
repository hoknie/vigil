use vigil_view::{Facet, Pane, RowKey, Section, Showing};

use super::agreed::the_index_lists_what_the_rows_list;
use super::scaled::many_launches;
use crate::views::WhatHasRunHere;

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

fn narrowings() -> Vec<Vec<Facet>> {
    vec![
        Vec::new(),
        vec![Facet::new("user", "alice")],
        vec![Facet::new("user", "root")],
        vec![Facet::new("user", "login 1000")],
        vec![Facet::new("program", "/usr/bin/nc.openbsd")],
        vec![
            Facet::new("user", "root"),
            Facet::new("program", "/usr/bin/id"),
        ],
        vec![Facet::new("program", "/usr/bin/nothing")],
    ]
}

#[test]
fn every_search_and_every_sort_of_many_launches_lists_from_the_index_what_the_reading_lists() {
    let reading = many_launches(30);
    let pane = pane();
    assert!(
        reading.items.len() >= 150,
        "a sample of six rows has no ties to break and proves nothing about the order they \
         are broken in"
    );
    assert!(
        reading
            .items
            .keys()
            .filter(|key| key.starts_with("launches|"))
            .count()
            > 1,
        "the rows about the reading are ordered among themselves too, and one of them orders \
         nothing"
    );

    for only in narrowings() {
        the_index_lists_what_the_rows_list(
            pane.as_ref(),
            &reading,
            Showing::default().narrowing(&only),
        );
    }
}

#[test]
fn a_list_narrowed_to_a_person_or_a_program_is_narrowed_while_the_index_is_built() {
    let reading = many_launches(8);
    let pane = pane();

    for only in narrowings() {
        let showing = Showing::default().narrowing(&only);
        let index = pane
            .index(&reading, &showing)
            .expect("this pane builds an index");

        let indexed: Vec<RowKey> = (0..index.len()).map(|at| index.row(at).clone()).collect();

        assert_eq!(
            indexed,
            pane.rows(&reading, &showing),
            "{only:?}: the console builds the index once per narrowing and answers only the \
             search and the sort from it, so a facet the index ignored would be ignored on \
             every keystroke"
        );
    }
}
