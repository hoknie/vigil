mod listing;
mod suite;
mod tallying;
mod views;

#[cfg(test)]
mod tests;

pub use listing::{
    the_index_lists_every_search_and_sort_as_the_rows_do,
    the_index_lists_every_search_and_sort_as_the_rows_do_also,
    the_index_lists_every_search_and_sort_as_the_rows_do_in,
};
pub use suite::{
    a_pane_draws_a_cell_for_every_column_it_declares, a_pane_reads_the_snapshot_it_says_it_reads,
    a_pane_says_what_to_write_over_it_and_over_the_row_it_opens,
    a_pane_that_draws_a_graph_draws_one_under_every_row_and_never_off_the_side,
    a_pane_that_gathers_rows_starts_with_every_one_of_them_put_away,
    a_pane_that_keeps_a_history_has_something_to_say_under_every_row,
    every_row_the_pane_offers_is_in_the_reading, nothing_is_shown_twice_under_one_key, run_all,
    what_the_pane_shows_of_a_row_is_more_than_the_row_itself,
};
pub use tallying::{
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says,
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_also,
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in,
};
pub use views::{every_view, every_view_answers_from_its_index_and_its_counts_as_from_the_reading};
