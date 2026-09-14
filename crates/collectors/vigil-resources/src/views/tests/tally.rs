use vigil_view::conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in;
use vigil_view::{Pane, RowKey, Section, Showing};

use super::scaled::many_filesystems;
use crate::views::TheHostAndItsFiles;

fn pane() -> Box<dyn Pane> {
    TheHostAndItsFiles.panes().remove(0)
}

#[test]
fn the_footer_of_many_filesystems_says_from_the_listed_rows_what_it_says_from_the_reading() {
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
        pane().as_ref(),
        &many_filesystems(40),
        Showing::default(),
    );
}

#[test]
fn the_filesystem_named_as_the_fullest_does_not_change_when_the_reader_turns_the_list_round() {
    let reading = many_filesystems(12);
    let pane = pane();
    let counts = pane
        .counts(&reading, &Showing::default())
        .expect("this pane counts");
    let mut rows: Vec<RowKey> = pane.rows(&reading, &Showing::default());

    let read_in_order = pane.tally_listed(&reading, &Showing::default(), &rows, &counts);
    rows.reverse();
    let turned_round = pane.tally_listed(&reading, &Showing::default(), &rows, &counts);

    assert_eq!(
        read_in_order, turned_round,
        "two filesystems with the same room left are told apart by their key and not by where \
         the reader's sort put them, or sorting the list would rename the fullest one"
    );
}
