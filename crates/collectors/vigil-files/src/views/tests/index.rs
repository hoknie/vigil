use vigil_view::conformance::the_index_lists_every_search_and_sort_as_the_rows_do_in;
use vigil_view::{Pane, RowKey, Section, Showing};

use super::scaled::many_files;
use crate::views::TheHostAndItsFiles;

fn pane() -> Box<dyn Pane> {
    TheHostAndItsFiles.panes().remove(0)
}

#[test]
fn every_search_and_every_sort_of_many_watched_paths_lists_from_the_index_what_the_reading_lists() {
    let reading = many_files(40);
    let pane = pane();
    assert!(
        reading.items.len() >= 200,
        "a sample of five rows has no ties to break and proves nothing about the order they \
         are broken in"
    );
    assert_eq!(
        pane.index(&reading, &Showing::default())
            .expect("a pane that builds no index is walked whole on every keystroke")
            .len(),
        pane.rows(&reading, &Showing::default()).len(),
        "the index holds every row the list shows before anything is searched for"
    );

    the_index_lists_every_search_and_sort_as_the_rows_do_in(
        pane.as_ref(),
        &reading,
        Showing::default(),
    );
}

#[test]
fn the_index_holds_the_watched_paths_in_the_order_they_are_listed_before_anything_is_chosen() {
    let reading = many_files(12);
    let pane = pane();
    let index = pane
        .index(&reading, &Showing::default())
        .expect("this pane builds an index");

    let indexed: Vec<RowKey> = (0..index.len()).map(|at| index.row(at).clone()).collect();

    assert_eq!(
        indexed,
        pane.rows(&reading, &Showing::default()),
        "two rows equal in a column keep the order they were pushed in, so that order has to \
         be the one the list is read in"
    );
}
