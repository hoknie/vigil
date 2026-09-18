use vigil_model::{Controlling, ProtocolError, Response, Rfc3339};

use super::Shared;
use crate::helpers::uuid7;
use crate::units::{READING, carry_out, findings};

const OFF: &str = "This agent starts and stops nothing on this host. Controlling what this host \
                   starts by itself from the console is off until units.from_the_console in the \
                   persistence block says otherwise (collectors/persistence.yaml, or vigil.yaml \
                   on a host of the former layout), and the daemon reads that key once, at \
                   start-up.";

pub fn control(
    keys: &[String],
    controlling: Controlling,
    shared: &Shared,
    now: Rfc3339,
) -> Response {
    if !shared.with(|state| state.units_from_the_console()) {
        return refused(ProtocolError::NOT_ALLOWED, OFF);
    }
    if keys.is_empty() {
        return refused(
            ProtocolError::NOTHING_TO_ACT_ON,
            "no unit, timer or cron job was named",
        );
    }

    let reading = shared.with(|state| state.snapshot(READING).cloned());
    let report = carry_out(keys, controlling, reading.as_ref(), now);
    let raised = findings(&report, &mut uuid7::mint);
    shared.with(|state| {
        state.record_what_the_console_did(&raised);
        state.ask_for_a_reading(READING);
    });

    for one in &report.controlled {
        eprintln!(
            "  console asked to {} {} — {}",
            controlling.as_str(),
            one.key,
            one.said
        );
    }

    Response::Controlled {
        report: Box::new(report),
    }
}

fn refused(code: &str, message: &str) -> Response {
    Response::Error {
        error: ProtocolError::new(code, message),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-16T10:00:00.000Z".to_string()
    }

    fn nginx() -> Vec<String> {
        vec!["unit|nginx.service".to_string()]
    }

    #[test]
    fn an_agent_nobody_switched_this_on_for_refuses_and_names_the_file_that_would() {
        for state in [
            fixture::state(),
            fixture::state_that_may_kill(),
            fixture::state_that_may_change(),
        ] {
            let shared = Shared::new(state);

            match control(&nginx(), Controlling::Stop, &shared, now()) {
                Response::Error { error } => {
                    assert_eq!(error.code, ProtocolError::NOT_ALLOWED);
                    assert!(
                        error.message.contains("units.from_the_console")
                            && error.message.contains("collectors/persistence.yaml"),
                        "the refusal names the key and the file it lives in: {error}"
                    );
                }
                other => panic!(
                    "it answered {other:?}: a host where a program may be stopped or an \
                     account changed has not thereby agreed that its services may be \
                     disabled"
                ),
            }
            assert!(
                shared
                    .with(|state| state.take_what_the_console_raised())
                    .is_empty()
            );
            assert!(
                shared
                    .with(|state| state.take_readings_asked_for())
                    .is_empty(),
                "an agent that did nothing has nothing new to read"
            );
        }
    }

    #[test]
    fn an_ask_with_nothing_in_it_is_refused_rather_than_answered_with_an_empty_report() {
        let shared = Shared::new(fixture::state_that_may_control());

        match control(&[], Controlling::Stop, &shared, now()) {
            Response::Error { error } => assert_eq!(error.code, ProtocolError::NOTHING_TO_ACT_ON),
            other => panic!("it answered {other:?}"),
        }
    }

    #[test]
    fn every_ask_answered_asks_for_what_starts_by_itself_to_be_read_again_even_a_refused_one() {
        let shared = Shared::new(fixture::state_that_may_control());

        control(&nginx(), Controlling::Stop, &shared, now());

        assert_eq!(
            shared.with(|state| state.take_readings_asked_for()),
            vec![READING.to_string()],
            "an ask that stopped half-way has changed the host too, and the screen a person \
             is looking at must not go on showing a unit as running"
        );
    }

    #[test]
    fn what_the_agent_did_is_in_its_own_findings_before_the_console_is_answered() {
        let shared = Shared::new(fixture::state_that_may_control());

        let answer = control(&nginx(), Controlling::Stop, &shared, now());

        match answer {
            Response::Controlled { report } => {
                assert_eq!(report.done(), 0, "{report:?}");
                assert!(
                    report.controlled[0].said.contains("has not read"),
                    "{report:?}"
                );
            }
            other => panic!("it answered {other:?}"),
        }
        let raised = shared.with(|state| state.take_what_the_console_raised());
        assert_eq!(raised.len(), 1);
        assert_eq!(raised[0].kind.as_str(), "agent.unit.control_refused");
        assert_eq!(
            raised[0].finding_key,
            "agent.unit.control|unit|nginx.service"
        );
    }
}
