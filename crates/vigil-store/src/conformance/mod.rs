mod suite;

pub use suite::{
    a_collector_with_no_history_has_no_baseline,
    a_different_kind_about_the_same_object_is_not_a_repeat,
    a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it,
    a_repeat_raises_the_counter_and_keeps_the_first_sighting,
    a_store_says_what_it_is_holding_and_what_it_threw_away, baselines_are_kept_per_collector,
    finding, open_findings_come_back_newest_first_and_within_the_limit,
    pruning_drops_the_old_and_keeps_the_rest, run_all, snapshot, the_baseline_is_the_last_reading,
    what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it,
};
