use vigil_view::{Section, Showing};

use super::scaled::{many_things_started, pulled_in_twice};
use super::tallied::the_footer_from_the_counts_says_what_the_footer_from_the_reading_says;
use crate::fixture::persistence;
use crate::views::WhatStartsByItself;

const TREE: &str = "tree";

#[test]
fn the_footer_of_every_list_as_a_list_or_as_a_tree_says_from_the_counts_what_it_says_from_the_reading()
 {
    for reading in [persistence(), pulled_in_twice(), many_things_started(20)] {
        for pane in WhatStartsByItself.panes() {
            if !pane.shown(&reading) {
                continue;
            }
            for showing in [Showing::default(), Showing::default().arranged(TREE)] {
                the_footer_from_the_counts_says_what_the_footer_from_the_reading_says(
                    pane.as_ref(),
                    &reading,
                    showing,
                );
            }
        }
    }
}

#[test]
fn the_tree_footer_explains_the_plus_sign_only_while_a_listed_unit_carries_one() {
    let reading = pulled_in_twice();
    let units = WhatStartsByItself
        .panes()
        .into_iter()
        .find(|pane| pane.name() == "units")
        .expect("the units pane");
    let counts = units
        .counts(&reading, &Showing::default().arranged(TREE))
        .expect("this pane counts");

    let every = Showing::default().arranged(TREE);
    let rows = units.rows(&reading, &every);
    let footer = units.tally_listed(&reading, &every, &rows, &counts);

    assert!(
        footer.contains("+N means"),
        "the sample holds a unit pulled in from two places, and a +1 nobody explains reads as \
         a count of something else: {footer}"
    );

    let nothing = Showing::searching("nothing of the sort").arranged(TREE);
    let footer = units.tally_listed(&reading, &nothing, &[], &counts);
    assert!(
        !footer.contains("+N means"),
        "with no unit listed there is no plus sign to explain: {footer}"
    );
}
