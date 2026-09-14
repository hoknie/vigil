use vigil_view::{Section, Showing};

use super::scaled::many_files;
use super::tallied::the_footer_from_the_counts_says_what_the_footer_from_the_reading_says;
use crate::views::TheHostAndItsFiles;

#[test]
fn the_footer_of_many_watched_paths_says_from_the_listed_rows_what_it_says_from_the_reading() {
    let pane = TheHostAndItsFiles.panes().remove(0);

    the_footer_from_the_counts_says_what_the_footer_from_the_reading_says(
        pane.as_ref(),
        &many_files(40),
        Showing::default(),
    );
}
