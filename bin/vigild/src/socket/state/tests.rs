use vigil_collect::Health;
use vigil_model::{CollectorState, Snapshot};

use crate::socket::fixture;
use crate::types::Startup;

use super::State;

#[test]
fn a_collector_that_could_not_read_is_degraded_and_keeps_its_last_reading() {
    let mut state = fixture::state();
    state.record_reading(fixture::reading(fixture::snapshot()));

    state.record_failure(
        "ports",
        "2026-09-09T09:00:30.000Z".into(),
        "not permitted to read /proc/net/tcp",
    );

    let agent = state.agent();
    assert_eq!(agent.collectors[0].state, CollectorState::Degraded);
    assert_eq!(agent.collectors[0].failures, 1);
    assert_eq!(agent.collectors[0].readings, 1);
    assert!(
        state.snapshot("ports").is_some(),
        "losing the privilege to look is not a host with nothing on it"
    );
}

fn with_launches_switched_off() -> State {
    State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: fixture::host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), 30u32)].into_iter().collect(),
            killing_from_the_console: false,
        },
        &[("ports", Health::Ok)],
        &[],
        &["launches".to_string()],
    )
}

#[test]
fn the_answer_names_the_configuration_this_daemon_was_started_with() {
    let agent = with_launches_switched_off().agent();

    assert_eq!(
        agent.configuration_path.as_deref(),
        Some("/etc/vigil/vigil.yaml"),
        "the console silences a finding by editing that file, and a daemon started with \
         another one would have the console write where nobody reads"
    );
}

#[test]
fn a_collector_switched_off_in_the_configuration_has_a_row_of_its_own() {
    let agent = with_launches_switched_off().agent();

    let off = agent
        .collectors
        .iter()
        .find(|collector| collector.name == "launches")
        .expect("the collector that is off is still in the table");
    assert_eq!(off.state, CollectorState::Off);
    assert_eq!(off.readings, 0);
    assert_eq!(off.failures, 0, "off is not a collector that failed");
    let reason = off.reason.as_deref().unwrap_or_default();
    assert!(reason.contains("`collectors:`"), "{reason}");
    assert!(reason.contains("what people run"), "{reason}");
    assert!(reason.contains("not failing"), "{reason}");
}

#[test]
fn what_is_switched_off_is_said_once_and_not_also_among_the_limitations() {
    let agent = with_launches_switched_off().agent();

    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("launches") || line.contains("switched off")),
        "the switched-off collector is a row now, not a sentence: {:?}",
        agent.limitations
    );
}

#[test]
fn a_snapshot_is_never_offered_for_a_collector_that_is_off() {
    let state = with_launches_switched_off();

    assert!(!state.knows_collector("launches"));
    assert_eq!(state.collector_names(), vec!["ports".to_string()]);
}

#[test]
fn a_degraded_collector_keeps_the_reason_it_was_given_at_start() {
    let mut state = State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: fixture::host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), 30u32)].into_iter().collect(),
            killing_from_the_console: false,
        },
        &[("ports", Health::Degraded("run as root".into()))],
        &[],
        &[],
    );

    state.record_reading(fixture::reading(Snapshot::new(
        "ports",
        "2026-09-09T09:00:00.000Z",
    )));

    let agent = state.agent();
    assert_eq!(agent.collectors[0].state, CollectorState::Degraded);
    assert_eq!(agent.collectors[0].reason.as_deref(), Some("run as root"));
}

#[test]
fn what_the_console_is_shown_is_the_reading_the_rules_compared() {
    let mut state = fixture::state();
    let reading = fixture::snapshot();

    state.record_reading(fixture::reading(reading.clone()));

    assert_eq!(state.snapshot("ports"), Some(&reading));
    assert!(state.snapshot("files").is_none());
}

#[test]
fn a_failed_delivery_is_counted_where_an_operator_can_see_it() {
    let mut state = fixture::state();

    state.record_delivery(
        "ndjson",
        "2026-09-09T09:00:00.000Z".into(),
        Some("no space left on device".into()),
    );

    let reporter = &state.agent().reporters[0];
    assert_eq!(reporter.failures, 1);
    assert_eq!(reporter.deliveries, 0);
    assert_eq!(
        reporter.last_error.as_deref(),
        Some("no space left on device")
    );
}

#[test]
fn a_collector_that_was_repaired_stops_being_reported_as_broken() {
    let mut state = fixture::state();

    state.record_health(
        "ports",
        &Health::Degraded("cannot resolve socket owners".into()),
    );
    let degraded = state.agent();
    let row = degraded
        .collectors
        .iter()
        .find(|row| row.name == "ports")
        .expect("the row is there");
    assert_eq!(row.state, CollectorState::Degraded);
    assert_eq!(row.reason.as_deref(), Some("cannot resolve socket owners"));

    state.record_health("ports", &Health::Ok);
    let repaired = state.agent();
    let row = repaired
        .collectors
        .iter()
        .find(|row| row.name == "ports")
        .expect("the row is there");
    assert_eq!(row.state, CollectorState::Ok);
    assert_eq!(row.reason, None, "the complaint has to go with the state");
}

#[test]
fn a_switched_off_collector_is_never_asked_and_never_moved() {
    let mut state = fixture::state();
    state.record_health("launches", &Health::Ok);

    let row = state
        .agent()
        .collectors
        .into_iter()
        .find(|row| row.name == "launches");

    if let Some(row) = row.filter(|row| row.state == CollectorState::Off) {
        assert_eq!(row.state, CollectorState::Off);
    }
}

#[test]
fn the_limitations_are_published_rather_than_left_to_be_discovered() {
    let agent = fixture::state().agent();

    assert!(
        !agent.limitations.is_empty(),
        "this build still cannot do several things an operator would assume it does"
    );
    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("memory only") || line.contains("restart forgets")),
        "the store landed: a limitation that has gone must leave this list, or the screen \
         is telling an operator to distrust something that now works"
    );
    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("marked resolved") || line.contains("suppress")),
        "resolution and suppressions both landed: {:?}",
        agent.limitations
    );
    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("Nothing is buffered")),
        "the outgoing buffer landed: a screen that still tells an operator a failed delivery is \
         lost teaches them to distrust something that now works: {:?}",
        agent.limitations
    );
}

#[test]
fn an_agent_that_reports_its_store_stops_saying_that_it_does_not() {
    let mut state = fixture::state();
    assert!(
        state
            .agent()
            .limitations
            .iter()
            .any(|line| line.contains("local history")),
        "until the numbers are on the wire, the screen has to say so"
    );

    state.record_store(vigil_model::StoreStatus {
        records: vigil_model::Counted {
            held: 4,
            ceiling: 10_000,
        },
        ..vigil_model::StoreStatus::default()
    });

    let agent = state.agent();
    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("local history")),
        "{:?}",
        agent.limitations
    );
    assert_eq!(agent.store.expect("a store").records.held, 4);
}

#[test]
fn an_agent_that_reports_its_buffers_stops_saying_that_it_does_not() {
    let mut state = fixture::state();
    assert!(
        state
            .agent()
            .limitations
            .iter()
            .any(|line| line.contains("waiting for a receiver")),
        "until the numbers are on the wire, the screen has to say so"
    );

    state.record_buffers(vec![vigil_model::BufferStatus {
        receiver: "ndjson".into(),
        pending: 12,
        pending_ceiling: 500,
        ..vigil_model::BufferStatus::default()
    }]);

    let agent = state.agent();
    assert!(
        !agent
            .limitations
            .iter()
            .any(|line| line.contains("waiting for a receiver")),
        "{:?}",
        agent.limitations
    );
    assert_eq!(agent.buffers.expect("the buffers").len(), 1);
}

#[test]
fn an_agent_with_two_receivers_of_one_kind_answers_with_two_rows() {
    let mut state = fixture::state();

    state.record_buffers(vec![
        vigil_model::BufferStatus {
            receiver: "ndjson".into(),
            pending: 0,
            ..vigil_model::BufferStatus::default()
        },
        vigil_model::BufferStatus {
            receiver: "ndjson-2".into(),
            pending: 41,
            ..vigil_model::BufferStatus::default()
        },
    ]);

    let buffers = state.agent().buffers.expect("the buffers");

    assert_eq!(buffers.len(), 2);
    assert_ne!(
        buffers[0].receiver, buffers[1].receiver,
        "two receivers hold two files, and one row for both hides whichever is behind"
    );
    assert_eq!(buffers[1].pending, 41);
}

#[test]
fn an_agent_with_no_receivers_at_all_says_so_instead_of_saying_nothing() {
    let mut state = fixture::state();

    state.record_buffers(Vec::new());

    let agent = state.agent();
    assert_eq!(
        agent.buffers.as_deref(),
        Some(&[][..]),
        "a daemon configured with no receiver is a normal daemon, and a console reading that \
         as `nothing said` would show it as an older build"
    );
}

#[test]
fn the_socket_answers_without_waiting_on_the_journal() {
    let mut state = fixture::state();

    state.record_store(vigil_model::StoreStatus {
        records: vigil_model::Counted {
            held: 1_284,
            ceiling: 10_000,
        },
        journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
        ..vigil_model::StoreStatus::default()
    });

    let agent = state.agent();

    let store = agent.store.expect("the numbers the watch left here");
    assert_eq!(store.records.held, 1_284);
    assert_eq!(
        store.journal_path.as_deref(),
        Some("/var/lib/vigil/findings/journal.ndjson"),
        "the console is answered from what the watch put here, not from a read of the journal"
    );
}

#[test]
fn status_says_when_each_collector_reads_next_and_how_many_slots_it_missed() {
    let mut state = fixture::state();
    let mut reading = fixture::reading(fixture::snapshot());
    reading.every_seconds = 30;
    reading.next_run_at = "2026-09-09T09:00:30.000Z".into();
    reading.skipped = 2;

    state.record_reading(reading);

    let ports = &state.agent().collectors[0];
    assert_eq!(ports.every_seconds, Some(30));
    assert_eq!(
        ports.next_run_at.as_deref(),
        Some("2026-09-09T09:00:30.000Z")
    );
    assert_eq!(
        ports.skipped, 2,
        "a slot the host slept through is a number, not a silence"
    );
}

#[test]
fn a_collector_that_has_not_read_yet_still_says_how_often_it_is_going_to() {
    let ports = &fixture::state().agent().collectors[0];

    assert_eq!(ports.every_seconds, Some(30));
    assert_eq!(
        ports.next_run_at, None,
        "the daemon has not read once; there is nothing to date"
    );
}

#[test]
fn a_collector_that_is_switched_off_is_given_no_period_at_all() {
    let off = with_launches_switched_off().agent();

    let launches = off
        .collectors
        .iter()
        .find(|collector| collector.name == "launches")
        .expect("the collector that is off is still in the table");
    assert_eq!(
        launches.every_seconds, None,
        "a period for something that does not run would be a promise nobody keeps"
    );
}

#[test]
fn the_cost_of_the_agent_reaches_the_console_or_says_it_was_not_measured() {
    let mut state = fixture::state();

    state.record_budget(Some(0.02), None);

    let agent = state.agent();
    assert_eq!(agent.budget.duty_percent, Some(0.02));
    assert_eq!(
        agent.budget.resident_kb, None,
        "not measured here is not measured as zero"
    );
}

#[test]
fn the_summary_of_the_list_counts_the_journal_and_this_run_as_one_history() {
    let mut state = fixture::state();

    state.recall_findings(vec![fixture::finding("from-the-journal")], 40);
    state.record_findings(&[fixture::finding("from-this-run")]);

    let findings = state.agent().findings;
    assert_eq!(findings.retained, 2);
    assert_eq!(
        findings.total, 41,
        "forty in the journal and one raised since is forty-one things that happened"
    );
    assert_eq!(
        findings.dropped, 39,
        "thirty-nine are on disk and off the screen, and the reader is owed that number"
    );
    assert_eq!(
        findings.retained as u64 + findings.dropped,
        findings.total,
        "held plus out of reach is everything, or one of the three numbers is wrong"
    );
}
