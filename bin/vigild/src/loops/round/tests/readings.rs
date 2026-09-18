use std::sync::atomic::Ordering;
use std::time::Instant;

use vigil_collect::Health;
use vigil_model::CollectorState;

use super::harness::{raised, shown, snapshot, watching};
use crate::types::Said;

#[test]
fn a_collector_that_reads_again_is_well_on_that_reading_and_not_five_minutes_later() {
    let mut it = watching(
        "recovered",
        Health::Degraded("the owner of one socket could not be resolved".into()),
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([(
        "network",
        "degraded:the owner of one socket could not be resolved".to_string(),
    )]);
    it.round.read(0, &mut said);
    *it.collector.health.lock().expect("not poisoned") = Health::Ok;

    it.round.read(0, &mut said);

    assert_eq!(
        shown(&it.round),
        CollectorState::Ok,
        "the reading that found the change knows the collector is well; waiting five minutes to \
         say so leaves a person looking at a red mark and at the findings that disprove it"
    );
    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "and the finding that closes the complaint is raised there and then: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_reading_the_console_asked_for_is_taken_at_once_and_the_period_starts_again_from_it() {
    let mut it = watching(
        "asked",
        Health::Ok,
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    it.round.schedule.advance(0, Instant::now());
    it.round
        .shared
        .with(|state| state.ask_for_a_reading("network"));

    it.round.read_what_the_console_asked_for(&mut said);

    assert!(
        it.round.shared.with(|state| state
            .snapshot("network")
            .is_some_and(|reading| reading.items.contains_key("tcp|0.0.0.0:4444"))),
        "a person who just changed the host is shown the host as it is now, not as it was \
         when the period last came round"
    );
    assert!(
        it.round.schedule.waiting(0, Instant::now()) > std::time::Duration::from_secs(29),
        "and the next reading is a whole period after this one, not the slot that was due anyway"
    );

    it.round.read_what_the_console_asked_for(&mut said);
    assert!(
        it.collector
            .readings
            .lock()
            .expect("not poisoned")
            .is_empty(),
        "both readings were taken, and a turn with nothing asked for reads nothing more"
    );
}

#[test]
fn a_reading_asked_for_a_collector_this_agent_does_not_watch_reads_nothing() {
    let mut it = watching("unwatched", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round
        .shared
        .with(|state| state.ask_for_a_reading("users"));

    it.round.read_what_the_console_asked_for(&mut said);

    assert_eq!(
        it.collector.readings.lock().expect("not poisoned").len(),
        1,
        "users is switched off on this host, and asking for it is not a reason to read network"
    );
}

#[test]
fn a_reading_that_found_nothing_does_not_pay_for_a_second_look_at_the_collector() {
    let mut it = watching(
        "quiet",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    let after_the_baseline = it.collector.asked.load(Ordering::Relaxed);

    it.round.read(0, &mut said);

    assert_eq!(
        it.collector.asked.load(Ordering::Relaxed),
        after_the_baseline,
        "asking a collector how it is costs as much as a reading on the collectors where it \
         walks /proc twice; on a host where nothing moved, nothing has to be asked"
    );
}

#[test]
fn a_collector_that_could_not_read_and_then_read_is_looked_at_again_without_waiting() {
    let mut it = watching("failed", Health::Ok, Vec::new());
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    let after_the_failure = it.collector.asked.load(Ordering::Relaxed);
    assert_eq!(
        shown(&it.round),
        CollectorState::Degraded,
        "a collector that could not read is not a collector reporting an empty host"
    );
    *it.collector.readings.lock().expect("not poisoned") = vec![snapshot(&[443])];

    it.round.read(0, &mut said);

    assert!(
        it.collector.asked.load(Ordering::Relaxed) > after_the_failure,
        "a collector that failed and read again has changed state, whatever its snapshot says"
    );
    assert_eq!(shown(&it.round), CollectorState::Ok);
}

#[test]
fn a_reading_of_a_collector_that_is_still_well_says_nothing_about_its_health() {
    let mut it = watching(
        "unchanged",
        Health::Ok,
        vec![snapshot(&[443, 4444]), snapshot(&[443])],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);

    it.round.read(0, &mut said);

    assert!(
        raised(&it.round)
            .iter()
            .all(|finding| !finding.kind.as_str().starts_with("agent.collector")),
        "a change on the host is not news about the agent: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.title.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        raised(&it.round).len(),
        1,
        "and the finding about the port that appeared is the only one"
    );
}
