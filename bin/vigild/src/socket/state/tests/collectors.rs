use vigil_collect::Health;
use vigil_model::{CollectorState, Snapshot};

use crate::socket::{State, fixture};
use crate::types::Startup;

#[test]
fn a_collector_that_could_not_read_is_degraded_and_keeps_its_last_reading() {
    let mut state = fixture::state();
    state.record_reading(fixture::reading(fixture::snapshot()));

    state.record_failure(
        "network",
        "2026-09-09T09:00:30.000Z".into(),
        "not permitted to read /proc/net/tcp",
    );

    let agent = state.agent();
    assert_eq!(agent.collectors[0].state, CollectorState::Degraded);
    assert_eq!(agent.collectors[0].failures, 1);
    assert_eq!(agent.collectors[0].readings, 1);
    assert!(
        state.snapshot("network").is_some(),
        "losing the privilege to look is not a host with nothing on it"
    );
}

#[test]
fn a_reading_asked_for_twice_before_the_loop_looks_is_taken_once_and_then_not_again() {
    let mut state = fixture::state();

    state.ask_for_a_reading("users");
    state.ask_for_a_reading("users");

    assert_eq!(
        state.take_readings_asked_for(),
        vec!["users".to_string()],
        "two changes saved in the same second want one reading of the accounts, not two"
    );
    assert!(
        state.take_readings_asked_for().is_empty(),
        "a reading taken is not asked for again at every turn of the loop"
    );
}

#[test]
fn a_degraded_collector_keeps_the_reason_it_was_given_at_start() {
    let mut state = State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: fixture::host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("network".to_string(), 30u32)].into_iter().collect(),
            killing_from_the_console: false,
            accounts_from_the_console: false,
            units_from_the_console: false,
        },
        &[("network", Health::Degraded("run as root".into()))],
        &[],
        &[],
    );

    state.record_reading(fixture::reading(Snapshot::new(
        "network",
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

    assert_eq!(state.snapshot("network"), Some(&reading));
    assert!(state.snapshot("files").is_none());
}

#[test]
fn a_collector_that_was_repaired_stops_being_reported_as_broken() {
    let mut state = fixture::state();

    state.record_health(
        "network",
        &Health::Degraded("cannot resolve socket owners".into()),
    );
    let degraded = state.agent();
    let row = degraded
        .collectors
        .iter()
        .find(|row| row.name == "network")
        .expect("the row is there");
    assert_eq!(row.state, CollectorState::Degraded);
    assert_eq!(row.reason.as_deref(), Some("cannot resolve socket owners"));

    state.record_health("network", &Health::Ok);
    let repaired = state.agent();
    let row = repaired
        .collectors
        .iter()
        .find(|row| row.name == "network")
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
fn status_says_when_each_collector_reads_next_and_how_many_slots_it_missed() {
    let mut state = fixture::state();
    let mut reading = fixture::reading(fixture::snapshot());
    reading.every_seconds = 30;
    reading.next_run_at = "2026-09-09T09:00:30.000Z".into();
    reading.skipped = 2;

    state.record_reading(reading);

    let network = &state.agent().collectors[0];
    assert_eq!(network.every_seconds, Some(30));
    assert_eq!(
        network.next_run_at.as_deref(),
        Some("2026-09-09T09:00:30.000Z")
    );
    assert_eq!(
        network.skipped, 2,
        "a slot the host slept through is a number, not a silence"
    );
}

#[test]
fn a_collector_that_has_not_read_yet_still_says_how_often_it_is_going_to() {
    let network = &fixture::state().agent().collectors[0];

    assert_eq!(network.every_seconds, Some(30));
    assert_eq!(
        network.next_run_at, None,
        "the daemon has not read once; there is nothing to date"
    );
}

fn watching(collectors: &[(&str, Health)]) -> State {
    State::new(
        Startup {
            configuration_path: "/etc/vigil/vigil.yaml".to_string(),
            host: fixture::host(),
            started_at: "2026-11-22T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: collectors
                .iter()
                .map(|(name, _)| (name.to_string(), 30u32))
                .collect(),
            killing_from_the_console: false,
            accounts_from_the_console: false,
            units_from_the_console: false,
        },
        collectors,
        &[],
        &[],
    )
}

#[test]
fn a_limited_collector_is_answered_as_limited_with_what_it_is_not_shown() {
    let mut state = watching(&[("network", Health::Ok)]);

    state.record_limit(
        "network",
        Some("the sockets of every other account are not seen"),
    );

    let row = &state.agent().collectors[0];
    assert_eq!(row.state, CollectorState::Limited);
    assert_eq!(row.reason, None);
    assert_eq!(
        row.limit.as_deref(),
        Some("the sockets of every other account are not seen")
    );
}

#[test]
fn a_failure_of_a_limited_collector_is_answered_as_the_failure_and_keeps_the_limit() {
    let mut state = watching(&[("users", Health::Ok)]);
    state.record_limit("users", Some("/private/etc/sudoers is not readable"));

    state.record_health("users", &Health::Degraded("utmpx is not there".into()));

    let row = &state.agent().collectors[0];
    assert_eq!(row.state, CollectorState::Degraded);
    assert_eq!(row.reason.as_deref(), Some("utmpx is not there"));
    assert_eq!(
        row.limit.as_deref(),
        Some("/private/etc/sudoers is not readable")
    );

    state.record_health("users", &Health::Ok);
    assert_eq!(state.agent().collectors[0].state, CollectorState::Limited);
}

#[test]
fn a_reading_that_failed_turns_a_limited_collector_into_one_in_trouble() {
    let mut state = watching(&[("network", Health::Ok)]);
    state.record_limit(
        "network",
        Some("the sockets of every other account are not seen"),
    );

    state.record_failure(
        "network",
        "2026-11-22T08:00:30.000Z".into(),
        "sysctl refused",
    );

    assert_eq!(state.agent().collectors[0].state, CollectorState::Degraded);
}

#[test]
fn a_limit_that_went_away_leaves_the_collector_well() {
    let mut state = watching(&[("network", Health::Ok)]);
    state.record_limit(
        "network",
        Some("the sockets of every other account are not seen"),
    );

    state.record_limit("network", None);

    let row = &state.agent().collectors[0];
    assert_eq!(row.state, CollectorState::Ok);
    assert_eq!(row.limit, None);
}
