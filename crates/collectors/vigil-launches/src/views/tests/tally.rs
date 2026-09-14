use vigil_view::{Facet, Section, Showing};

use super::scaled::many_launches;
use super::tallied::the_footer_from_the_counts_says_what_the_footer_from_the_reading_says;
use crate::fixture::launches;
use crate::views::WhatHasRunHere;

#[test]
fn the_footer_of_many_launches_narrowed_or_not_says_from_the_counts_what_it_says_from_the_reading()
{
    let pane = WhatHasRunHere.panes().remove(0);
    let narrowings = [
        Vec::new(),
        vec![Facet::new("user", "alice")],
        vec![Facet::new("program", "/usr/bin/nc.openbsd")],
        vec![
            Facet::new("user", "root"),
            Facet::new("program", "/usr/bin/id"),
        ],
        vec![Facet::new("program", "/usr/bin/nothing")],
    ];

    for reading in [launches(), many_launches(30)] {
        for only in &narrowings {
            the_footer_from_the_counts_says_what_the_footer_from_the_reading_says(
                pane.as_ref(),
                &reading,
                Showing::default().narrowing(only),
            );
        }
    }
}
