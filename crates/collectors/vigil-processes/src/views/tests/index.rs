use vigil_view::{Pane, RowKey, Section, Showing};

use super::agreed::the_index_lists_what_the_rows_list;
use super::scaled::many_programs;
use crate::views::WhatHasRunHere;

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

#[test]
fn every_search_of_many_running_programs_lists_from_the_index_what_the_reading_lists() {
    let reading = many_programs(40);
    assert!(
        reading
            .items
            .keys()
            .filter(|key| key.starts_with("processes|"))
            .count()
            > 1
            && reading.items.len() >= 200,
        "a sample with one row about the reading and a handful of programs never shows the \
         rows about the reading kept ahead of a larger list"
    );

    the_index_lists_what_the_rows_list(pane().as_ref(), &reading, Showing::default());
}

#[test]
fn the_rows_about_the_reading_are_pushed_first_so_every_search_keeps_them_on_top() {
    let reading = many_programs(12);
    let pane = pane();
    let index = pane
        .index(&reading, &Showing::default())
        .expect("this pane builds an index");

    let indexed: Vec<RowKey> = (0..index.len()).map(|at| index.row(at).clone()).collect();

    assert_eq!(
        indexed,
        pane.rows(&reading, &Showing::default()),
        "this list is not sorted by the reader, so what a search finds is listed in the order \
         the rows were pushed, and that order is the one the list is read in"
    );
}
