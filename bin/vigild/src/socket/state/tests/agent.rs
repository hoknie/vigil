use crate::socket::fixture;

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
