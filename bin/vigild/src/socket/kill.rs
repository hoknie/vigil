use vigil_model::{Killing, ProtocolError, Response, Rfc3339};

use super::Shared;
use crate::helpers::uuid7;
use crate::killing::{READING, carry_out, findings};

const OFF: &str = "This agent does not close sockets. Killing from the console is off until \
                   vigil.yaml says otherwise, and the daemon reads that key once, at \
                   start-up.";

pub fn kill(sockets: &[String], killing: Killing, shared: &Shared, now: Rfc3339) -> Response {
    let (allowed, reading) = shared.with(|state| {
        (
            state.killing_from_the_console(),
            state.snapshot(READING).cloned(),
        )
    });

    if !allowed {
        return Response::Error {
            error: ProtocolError::new(ProtocolError::NOT_ALLOWED, OFF),
        };
    }
    if sockets.is_empty() {
        return Response::Error {
            error: ProtocolError::new(
                ProtocolError::NOTHING_TO_ACT_ON,
                "no socket was named to close",
            ),
        };
    }

    let report = carry_out(sockets, killing, reading.as_ref(), now);
    let raised = findings(&report, &mut uuid7::mint);
    shared.with(|state| state.record_a_kill(&raised));

    for one in &report.killed {
        eprintln!("  console asked to close {} — {}", one.key, one.said);
    }

    Response::Killed {
        report: Box::new(report),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-14T10:00:00.000Z".to_string()
    }

    #[test]
    fn an_agent_nobody_switched_this_on_for_refuses_and_names_the_file_that_would() {
        let shared = Shared::new(fixture::state());

        match kill(&["tcp|0.0.0.0:4444".into()], Killing::Kill, &shared, now()) {
            Response::Error { error } => {
                assert_eq!(error.code, ProtocolError::NOT_ALLOWED);
                assert!(error.message.contains("vigil.yaml"), "{error}");
            }
            other => panic!("it answered {other:?}"),
        }
    }

    #[test]
    fn an_ask_with_nothing_in_it_is_refused_rather_than_answered_with_an_empty_report() {
        let shared = Shared::new(fixture::state_that_may_kill());

        match kill(&[], Killing::Terminate, &shared, now()) {
            Response::Error { error } => {
                assert_eq!(error.code, ProtocolError::NOTHING_TO_ACT_ON)
            }
            other => panic!("it answered {other:?}"),
        }
    }

    #[test]
    fn what_the_agent_did_is_in_its_own_findings_before_the_console_is_answered() {
        let mut state = fixture::state_that_may_kill();
        state.record_reading(fixture::reading(fixture::snapshot()));
        let shared = Shared::new(state);

        let answer = kill(
            &["tcp|0.0.0.0:4444".into(), "tcp|nothing:1".into()],
            Killing::Terminate,
            &shared,
            now(),
        );

        assert!(matches!(answer, Response::Killed { .. }));
        let raised = shared.with(|state| state.take_what_a_kill_raised());
        assert_eq!(
            raised.len(),
            2,
            "both rows are in the journal: an agent that records only what worked cannot \
             answer 'was this asked for' at an incident"
        );
        assert!(
            shared
                .with(|state| state.take_what_a_kill_raised())
                .is_empty(),
            "taking the findings twice would report them twice"
        );
    }
}
