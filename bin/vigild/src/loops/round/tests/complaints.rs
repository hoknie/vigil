use vigil_collect::Health;

use super::harness::{open_in_the_journal, raised, snapshot, watching};
use crate::types::Said;

#[test]
fn a_reading_that_failed_is_a_finding_and_not_only_a_line_in_the_daemons_log() {
    let mut it = watching("failed-once", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);

    it.round.read(0, &mut said);

    let findings = raised(&it.round);
    let about_it = findings
        .iter()
        .find(|finding| finding.kind.as_str() == "agent.collector.degraded")
        .expect("a collector that cannot read has to say so where the findings go");
    assert_eq!(about_it.finding_key, "agent.collector|ports");
    assert!(
        about_it
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "error" && evidence.value.contains("ports")),
        "the finding carries the same error the screen shows: {:?}",
        about_it.evidence
    );
    assert!(
        about_it
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "cost" && evidence.value.contains("failed reading")),
        "and the same counters: {:?}",
        about_it.evidence
    );
}

#[test]
fn a_collector_that_fails_every_reading_raises_one_finding_and_not_one_a_tick() {
    let mut it = watching("failing", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);

    for _ in 0..10 {
        it.round.read(0, &mut said);
    }

    let about_it = raised(&it.round)
        .into_iter()
        .filter(|finding| finding.kind.as_str() == "agent.collector.degraded")
        .count();
    assert_eq!(
        about_it, 1,
        "a failure that repeats every tick is one thing wrong with the host, and a finding a \
         tick is a generator of rubbish"
    );
    assert_eq!(
        open_in_the_journal(&it.round)
            .expect("the finding is in the journal")
            .occurrences,
        1,
        "the same failure repeating is not a new sighting to write down either: what changes \
         the record is a reason that changed"
    );
}

#[test]
fn a_reading_that_went_through_closes_the_finding_about_the_ones_that_did_not() {
    let mut it = watching("failed-then-read", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);
    for _ in 0..3 {
        it.round.read(0, &mut said);
    }
    assert!(open_in_the_journal(&it.round).is_some(), "it stood first");

    *it.collector.readings.lock().expect("not poisoned") = vec![snapshot(&[443])];
    it.round.read(0, &mut said);

    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "a collector that read again closes what was raised when it could not: {:?}",
        raised(&it.round)
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        open_in_the_journal(&it.round).is_none(),
        "and the journal a restart reads has nothing open about it any more"
    );
}

#[test]
fn a_reason_that_changed_is_written_down_without_alerting_a_second_time() {
    let mut it = watching("two-reasons", Health::Ok, Vec::new());
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);

    said.failing("ports", "something else entirely");
    it.round.read(0, &mut said);

    assert_eq!(
        open_in_the_journal(&it.round)
            .expect("still open")
            .occurrences,
        2,
        "a reason that changed is written into the history of the same finding"
    );
    assert_eq!(
        raised(&it.round)
            .iter()
            .filter(|finding| finding.kind.as_str() == "agent.collector.degraded")
            .count(),
        1,
        "and it is not a second thing to tell a person about: one collector, one complaint"
    );
}

#[test]
fn a_complaint_a_previous_run_left_open_is_closed_by_the_first_reading_that_goes_through() {
    let mut it = watching("from-the-last-run", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("ports", "ok".to_string())]);
    said.opened(
        "ports",
        "the reading failed — ports: /proc/net/tcp: permission denied".to_string(),
    );

    it.round.read(0, &mut said);

    let closed = raised(&it.round)
        .into_iter()
        .find(|finding| finding.kind.as_str() == "agent.collector.recovered")
        .expect("the first reading that went through is the proof it is watching again");
    assert!(
        closed
            .evidence
            .iter()
            .any(|evidence| evidence.value.contains("permission denied")),
        "and it quotes the complaint it closes, which came out of the journal: {:?}",
        closed.evidence
    );
}

#[test]
fn a_collector_that_came_back_is_reported_once_and_not_on_every_reading_after_it() {
    let mut it = watching(
        "came-back",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    said.opened("ports", "degraded — one socket had no owner".to_string());

    it.round.read(0, &mut said);
    it.round.read(0, &mut said);
    it.round.read(0, &mut said);

    assert_eq!(
        raised(&it.round)
            .iter()
            .filter(|finding| finding.kind.as_str() == "agent.collector.recovered")
            .count(),
        1,
        "coming back happened once; a line a reading would be the noise the pair exists to avoid"
    );
}

#[test]
fn a_collector_that_goes_unwell_while_the_agent_runs_is_reported_and_then_closed() {
    let mut it = watching(
        "mid-run",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("ports", "ok".to_string())]);
    it.round.read(0, &mut said);

    *it.collector.health.lock().expect("not poisoned") =
        Health::Degraded("the owner of one socket could not be resolved".into());
    it.round.take_health(true, &mut said);
    let complained = raised(&it.round);
    *it.collector.health.lock().expect("not poisoned") = Health::Ok;
    it.round.take_health(true, &mut said);

    assert!(
        complained
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.degraded"),
        "a collector that stops seeing part of a host mid-run says so: {:?}",
        complained
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        raised(&it.round)
            .iter()
            .any(|finding| finding.kind.as_str() == "agent.collector.recovered"),
        "and the pass that finds it well again closes what that raised"
    );
    assert!(
        open_in_the_journal(&it.round).is_none(),
        "in the journal too, which is what the next start reads"
    );
}
