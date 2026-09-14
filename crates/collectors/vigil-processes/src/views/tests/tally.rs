use vigil_view::conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in;
use vigil_view::{Section, Showing};

use super::scaled::many_programs;
use crate::views::WhatHasRunHere;

#[test]
fn the_footer_of_many_running_programs_says_from_the_counts_what_it_says_from_the_reading() {
    let pane = WhatHasRunHere.panes().remove(0);

    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
        pane.as_ref(),
        &many_programs(40),
        Showing::default(),
    );
}
