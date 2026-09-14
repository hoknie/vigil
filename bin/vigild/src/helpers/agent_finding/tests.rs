use vigil_model::{CollectorStatus, Kind, KnownKind, Snapshot};
use vigil_store::Held;

use super::{
    budget_exceeded, budget_recovered, buffer_drained, buffer_dropping, collector_degraded,
    collector_failing, store_damaged,
};
use crate::budget::{CPU, Meter};
use crate::types::Reading;

fn expensive() -> Meter {
    let mut meter = Meter::default();
    meter.record(&Reading {
        collector: "ports",
        at: "2026-09-10T12:00:00.000Z".into(),
        duration_ms: 600,
        every_seconds: 30,
        next_run_at: "2026-09-10T12:00:30.000Z".into(),
        skipped: 0,
        snapshot: Snapshot::new("ports", "2026-09-10T12:00:00.000Z"),
    });
    meter
}

#[test]
fn the_finding_carries_the_schedule_line_that_fixes_it() {
    let finding = budget_exceeded(CPU, "This agent costs 2.00 % of one core", &expensive());

    let schedule = finding
        .evidence
        .iter()
        .find(|evidence| evidence.kind == "schedule")
        .expect("the way out is in the finding, not in a document");
    assert!(
        schedule.value.contains("schedule:\n  ports: 60"),
        "{schedule:?}"
    );
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "cost"
                && evidence.value.contains("ports")
                && evidence.value.contains("every 30 s")),
        "the table names the collector, its share and its period"
    );
}

#[test]
fn going_back_under_the_ceiling_closes_the_finding_rather_than_opening_a_second_one() {
    let opened = budget_exceeded(CPU, "over", &expensive());
    let closed = budget_recovered(CPU, "back under", "three checks in a row under the ceiling");

    assert_eq!(
        opened.finding_key, closed.finding_key,
        "the two are statements about one object"
    );
    assert_eq!(closed.kind, Kind::Known(KnownKind::AgentBudgetRecovered));
    assert_eq!(
        KnownKind::AgentBudgetRecovered.resolves(),
        Some(KnownKind::AgentBudgetExceeded)
    );
}

fn tried(readings: u64, failures: u64, last_error: &str) -> CollectorStatus {
    CollectorStatus {
        name: "launches".into(),
        state: vigil_model::CollectorState::Degraded,
        reason: None,
        last_run_at: Some("2026-09-12T09:00:00.000Z".into()),
        duration_ms: None,
        items: 0,
        readings,
        every_seconds: Some(30),
        next_run_at: None,
        skipped: 2,
        failures,
        last_error: Some(last_error.to_string()),
        baseline: false,
    }
}

#[test]
fn a_host_where_a_collector_cannot_run_and_one_where_it_just_broke_are_told_apart_in_the_proof() {
    let never = collector_degraded("launches", "auditd is not running on this host", true);
    let broke = collector_failing(
        "launches",
        &tried(
            41,
            3,
            "launches: /var/lib/vigil/audit-spool: permission denied",
        ),
    );

    assert_eq!(
        never.kind, broke.kind,
        "both say the same thing about the same object — the agent is not watching part of \
         this host — and a second kind for the second cause would be two findings about one \
         blind collector"
    );
    assert_eq!(never.finding_key, broke.finding_key);
    assert_ne!(
        never.title, broke.title,
        "but the two are different hosts to a person: one has nothing to fix, the other \
         broke a moment ago"
    );
    assert!(
        broke
            .evidence
            .iter()
            .any(|evidence| evidence.value.contains("read this host 41 time(s)")),
        "and the proof says it used to work: {:?}",
        broke.evidence
    );
    assert!(
        broke.evidence.iter().any(
            |evidence| evidence.kind == "error" && evidence.value.contains("permission denied")
        ),
        "quoting the error a person reads on the screen: {:?}",
        broke.evidence
    );
}

#[test]
fn a_collector_that_has_never_read_here_does_not_claim_that_something_just_broke() {
    let first_time = collector_failing("launches", &tried(0, 1, "launches: no such file"));

    assert!(
        first_time
            .evidence
            .iter()
            .any(|evidence| evidence.value.contains("has not completed a reading")),
        "{:?}",
        first_time.evidence
    );
    assert!(
        !first_time
            .evidence
            .iter()
            .any(|evidence| evidence.value.contains("something here changed")),
        "a reading that never once worked is not a host that changed a moment ago: {:?}",
        first_time.evidence
    );
}

#[test]
fn the_pair_about_a_buffer_is_two_statements_about_one_receiver() {
    let held = Held {
        records: vigil_store::Counted::new(2_000, 2_000),
        bytes: vigil_store::Counted::new(4 * 1024 * 1024, 4 * 1024 * 1024),
        dropped: 41,
        damaged: 0,
        oldest_at: Some("2026-09-10T09:00:00.000Z".into()),
    };

    let full = buffer_dropping("host-findings", 41, &held);
    let empty = buffer_drained("host-findings", 41);

    assert_eq!(full.finding_key, empty.finding_key);
    assert_eq!(
        KnownKind::AgentBufferDrained.resolves(),
        Some(KnownKind::AgentBufferDropping),
        "a buffer that empties has to close the finding that it was full, or the screen \
         carries it for the life of the host"
    );
    assert!(
        full.evidence
            .iter()
            .any(|evidence| evidence.value.contains("2026-09-10T09:00:00.000Z")),
        "and the finding says how old the oldest thing nobody has seen is: {:?}",
        full.evidence
    );
}

#[test]
fn the_part_of_the_store_that_could_not_be_read_is_named_rather_than_implied() {
    let history = store_damaged("findings", "the local findings history", 2);
    let outgoing = store_damaged("outgoing/syslog", "the outgoing buffer for syslog", 1);

    assert_ne!(
        history.finding_key, outgoing.finding_key,
        "two damaged files under one key is one of them hidden"
    );
    assert!(outgoing.title.contains("the outgoing buffer for syslog"));
}

#[test]
fn a_memory_ceiling_is_not_answered_with_a_reading_period() {
    let finding = budget_exceeded("resident", "This agent holds 70 MB", &expensive());

    assert!(
        !finding
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "schedule"),
        "reading less often does not give memory back"
    );
}
