use vigil_view::{Section, Showing};

use super::scaled::many_programs;
use super::tallied::the_footer_from_the_counts_says_what_the_footer_from_the_reading_says;
use crate::views::WhatHasRunHere;

#[test]
fn the_footer_of_many_running_programs_says_from_the_counts_what_it_says_from_the_reading() {
    let pane = WhatHasRunHere.panes().remove(0);

    the_footer_from_the_counts_says_what_the_footer_from_the_reading_says(
        pane.as_ref(),
        &many_programs(40),
        Showing::default(),
    );
}
