use vigil_view::{Emphasis, Pane, RowKey, Section, Showing, listed};

use super::agreed::the_index_lists_what_the_rows_list;
use super::scaled::many_things_started;
use crate::fixture::persistence;
use crate::views::WhatStartsByItself;

const TREE: &str = "tree";

fn units() -> Box<dyn Pane> {
    WhatStartsByItself
        .panes()
        .into_iter()
        .find(|pane| pane.name() == "units")
        .expect("the units pane")
}

#[test]
fn every_search_of_every_list_of_a_host_starting_many_things_lists_from_the_index_what_the_reading_lists()
 {
    let reading = many_things_started(25);
    assert!(
        reading.items.len() >= 200,
        "a sample of a few rows per list proves nothing about a list long enough to need an \
         index"
    );

    for pane in WhatStartsByItself.panes() {
        if !pane.shown(&reading) {
            continue;
        }
        the_index_lists_what_the_rows_list(pane.as_ref(), &reading, Showing::default());
        the_index_lists_what_the_rows_list(
            pane.as_ref(),
            &reading,
            Showing::default().arranged(TREE),
        );
    }
}

#[test]
fn the_units_drawn_as_a_tree_are_indexed_in_the_order_and_at_the_depth_they_are_drawn() {
    for reading in [persistence(), many_things_started(6)] {
        let pane = units();
        let showing = Showing::default().arranged(TREE);
        let index = pane
            .index(&reading, &showing)
            .expect("this pane builds an index");

        let indexed: Vec<RowKey> = (0..index.len()).map(|at| index.row(at).clone()).collect();

        assert!(
            indexed.iter().any(|row| row.depth > 0),
            "a tree with nothing under anything proves nothing about depth: {indexed:?}"
        );
        assert_eq!(
            indexed,
            pane.rows(&reading, &showing),
            "the rows are pushed in the order the tree is drawn, with their depth, so the order \
             they are kept in after a search is the tree's"
        );
    }
}

#[test]
fn a_search_in_the_tree_keeps_the_rows_it_finds_in_the_order_the_tree_draws_them() {
    let reading = many_things_started(6);
    let pane = units();

    for search in [
        "service",
        "TARGET",
        "nginx",
        "~003",
        "sh",
        "nothing of the sort",
    ] {
        let showing = Showing::searching(search).arranged(TREE);
        let index = pane
            .index(&reading, &showing)
            .expect("this pane builds an index");

        assert_eq!(
            listed(pane.as_ref(), &reading, &showing, &index, None).1,
            pane.rows(&reading, &showing),
            "{search:?}: the list builds the whole tree and then keeps the matching rows, and \
             keeping the matching rows of the tree-ordered index is the same thing"
        );
    }
}

#[test]
fn the_row_that_says_the_modules_were_not_readable_keeps_its_mark_when_it_comes_from_the_index() {
    let reading = persistence();
    let modules = WhatStartsByItself
        .panes()
        .into_iter()
        .find(|pane| pane.name() == "modules")
        .expect("the modules pane");
    let index = modules
        .index(&reading, &Showing::default())
        .expect("this pane builds an index");

    let unreadable = (0..index.len())
        .map(|at| index.row(at))
        .find(|row| row.key == "modules|unreadable")
        .expect("the sample could not read the modules");

    assert_eq!(
        unreadable.emphasis,
        Emphasis::Marked,
        "a row about the reading drawn plain from the index reads as one more module"
    );
}
