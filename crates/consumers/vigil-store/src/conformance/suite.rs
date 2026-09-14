use super::baselines::{
    a_collector_with_no_history_has_no_baseline,
    a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it, baselines_are_kept_per_collector,
    the_baseline_is_the_last_reading,
};
use super::findings::{
    a_different_kind_about_the_same_object_is_not_a_repeat,
    a_repeat_raises_the_counter_and_keeps_the_first_sighting, a_resolved_finding_stops_being_open,
    one_open_finding_is_found_by_its_object_and_its_kind,
    open_findings_come_back_newest_first_and_within_the_limit,
    resolving_something_that_was_never_recorded_is_not_an_error,
};
use super::retention::{
    a_store_says_how_much_of_its_history_is_still_open,
    a_store_says_what_it_is_holding_and_what_it_threw_away,
    pruning_drops_the_old_and_keeps_the_rest,
    what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it,
};
use crate::Store;

pub fn run_all(new_store: &dyn Fn() -> Box<dyn Store>) {
    a_collector_with_no_history_has_no_baseline(new_store().as_ref());
    the_baseline_is_the_last_reading(new_store().as_ref());
    baselines_are_kept_per_collector(new_store().as_ref());
    a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it(new_store().as_ref());
    a_repeat_raises_the_counter_and_keeps_the_first_sighting(new_store().as_ref());
    a_different_kind_about_the_same_object_is_not_a_repeat(new_store().as_ref());
    a_resolved_finding_stops_being_open(new_store().as_ref());
    one_open_finding_is_found_by_its_object_and_its_kind(new_store().as_ref());
    resolving_something_that_was_never_recorded_is_not_an_error(new_store().as_ref());
    open_findings_come_back_newest_first_and_within_the_limit(new_store().as_ref());
    pruning_drops_the_old_and_keeps_the_rest(new_store().as_ref());
    a_store_says_what_it_is_holding_and_what_it_threw_away(new_store().as_ref());
    a_store_says_how_much_of_its_history_is_still_open(new_store().as_ref());
    what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it(new_store().as_ref());
}
