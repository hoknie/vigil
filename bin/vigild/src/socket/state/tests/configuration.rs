use vigil_collect::Health;
use vigil_model::CollectorState;

use crate::Config;
use crate::socket::{State, fixture, switched_off_reasons};
use crate::types::Startup;

fn with_launches_switched_off() -> State {
    State::new(
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
        &[("network", Health::Ok)],
        &[],
        &switched_off_reasons(&Config::default(), &["launches".to_string()]),
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
    assert_eq!(state.collector_names(), vec!["network".to_string()]);
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
fn a_console_allowed_any_one_of_the_three_things_keeps_the_watching_loop_awake_for_it() {
    assert!(
        !with_launches_switched_off().console_may_act(),
        "a daemon nobody switched anything on for sleeps until its next reading is due"
    );

    for state in [
        fixture::state_that_may_kill(),
        fixture::state_that_may_change(),
        fixture::state_that_may_control(),
    ] {
        assert!(
            state.console_may_act(),
            "the loop rests a shorter while when the console may act, because a person who \
             stopped a unit is looking at a screen that must not go on saying it runs; a \
             switch missing from this answer is a screen that waits out the whole period"
        );
    }
}
