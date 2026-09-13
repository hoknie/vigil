mod suite;

#[cfg(test)]
mod tests;

pub use suite::{
    a_pane_draws_a_cell_for_every_column_it_declares, a_pane_reads_the_snapshot_it_says_it_reads,
    a_pane_says_what_to_write_over_it_and_over_the_row_it_opens,
    every_row_the_pane_offers_is_in_the_reading, nothing_is_shown_twice_under_one_key, run_all,
    what_the_pane_shows_of_a_row_is_more_than_the_row_itself,
};
