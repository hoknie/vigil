mod baselines;
mod findings;
mod retention;
mod samples;
mod suite;

pub use baselines::{
    a_collector_with_no_history_has_no_baseline,
    a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it, baselines_are_kept_per_collector,
    the_baseline_is_the_last_reading,
};
pub use findings::{
    a_different_kind_about_the_same_object_is_not_a_repeat,
    a_repeat_raises_the_counter_and_keeps_the_first_sighting,
    one_open_finding_is_found_by_its_object_and_its_kind,
    open_findings_come_back_newest_first_and_within_the_limit,
};
pub use retention::{
    a_store_says_how_much_of_its_history_is_still_open,
    a_store_says_what_it_is_holding_and_what_it_threw_away,
    pruning_drops_the_old_and_keeps_the_rest,
    what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it,
};
pub use samples::{finding, finding_of, snapshot};
pub use suite::run_all;
