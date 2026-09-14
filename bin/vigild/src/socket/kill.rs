use vigil_model::{KillTarget, Killing, ProtocolError, Response, Rfc3339};

use super::Shared;
use crate::helpers::uuid7;
use crate::killing::{carry_out, findings, reading_of};

const OFF: &str = "This agent does not close sockets or stop programs. Killing from the console \
                   is off until vigil.yaml says otherwise, and the daemon reads that key once, \
                   at start-up.";

pub fn kill(
    sockets: &[String],
    programs: &[String],
    killing: Killing,
    shared: &Shared,
    now: Rfc3339,
) -> Response {
    if !shared.with(|state| state.killing_from_the_console()) {
        return refused(ProtocolError::NOT_ALLOWED, OFF);
    }
    let (target, keys) = match (sockets.is_empty(), programs.is_empty()) {
        (false, false) => {
            return refused(
                ProtocolError::MALFORMED_REQUEST,
                "one ask names sockets or programs, not both: each is looked up in a reading \
                 of its own and reported under a finding of its own",
            );
        }
        (true, true) => {
            return refused(
                ProtocolError::NOTHING_TO_ACT_ON,
                "no socket was named to close and no program to stop",
            );
        }
        (true, false) => (KillTarget::Program, programs),
        (false, true) => (KillTarget::Socket, sockets),
    };

    let reading = shared.with(|state| state.snapshot(reading_of(target)).cloned());
    let report = carry_out(target, keys, killing, reading.as_ref(), now);
    let raised = findings(&report, &mut uuid7::mint);
    shared.with(|state| state.record_what_the_console_did(&raised));

    for one in &report.killed {
        eprintln!(
            "  console asked to {} {} — {}",
            match target {
                KillTarget::Socket => "close",
                KillTarget::Program => "stop",
            },
            one.key,
            one.said
        );
    }

    Response::Killed {
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
        "2026-09-14T10:00:00.000Z".to_string()
    }

    #[test]
    fn an_agent_nobody_switched_this_on_for_refuses_and_names_the_file_that_would() {
        let shared = Shared::new(fixture::state());

        for (sockets, programs) in [
            (vec!["tcp|0.0.0.0:4444".to_string()], Vec::new()),
            (Vec::new(), vec!["exec|/tmp/.x/nc|www-data".to_string()]),
        ] {
            match kill(&sockets, &programs, Killing::Kill, &shared, now()) {
                Response::Error { error } => {
                    assert_eq!(error.code, ProtocolError::NOT_ALLOWED);
                    assert!(error.message.contains("vigil.yaml"), "{error}");
                }
                other => panic!("it answered {other:?}"),
            }
        }
    }

    #[test]
    fn an_ask_with_nothing_in_it_is_refused_rather_than_answered_with_an_empty_report() {
        let shared = Shared::new(fixture::state_that_may_kill());

        match kill(&[], &[], Killing::Terminate, &shared, now()) {
            Response::Error { error } => {
                assert_eq!(error.code, ProtocolError::NOTHING_TO_ACT_ON)
            }
            other => panic!("it answered {other:?}"),
        }
    }

    #[test]
    fn an_ask_naming_sockets_and_programs_at_once_is_refused_before_anything_is_signalled() {
        let mut state = fixture::state_that_may_kill();
        state.record_reading(fixture::reading(fixture::snapshot()));
        let shared = Shared::new(state);

        match kill(
            &["tcp|0.0.0.0:4444".into()],
            &["exec|/tmp/.x/nc|www-data".into()],
            Killing::Terminate,
            &shared,
            now(),
        ) {
            Response::Error { error } => {
                assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST)
            }
            other => panic!("it answered {other:?}"),
        }
        assert!(
            shared
                .with(|state| state.take_what_the_console_raised())
                .is_empty()
        );
    }

    #[test]
    fn what_the_agent_did_is_in_its_own_findings_before_the_console_is_answered() {
        let mut state = fixture::state_that_may_kill();
        state.record_reading(fixture::reading(fixture::snapshot()));
        let shared = Shared::new(state);

        let answer = kill(
            &["tcp|0.0.0.0:4444".into(), "tcp|nothing:1".into()],
            &[],
            Killing::Terminate,
            &shared,
            now(),
        );

        assert!(matches!(answer, Response::Killed { .. }));
        let raised = shared.with(|state| state.take_what_the_console_raised());
        assert_eq!(
            raised.len(),
            2,
            "both rows are in the journal: an agent that records only what worked cannot \
             answer 'was this asked for' at an incident"
        );
        assert!(
            shared
                .with(|state| state.take_what_the_console_raised())
                .is_empty(),
            "taking the findings twice would report them twice"
        );
    }

    #[test]
    fn a_program_is_looked_up_in_the_reading_of_programs_and_journalled_as_a_program() {
        let mut state = fixture::state_that_may_kill();
        state.record_reading(fixture::reading_of(
            "processes",
            vigil_processes::fixture::processes(),
        ));
        let shared = Shared::new(state);

        let answer = kill(
            &[],
            &["exec|/usr/bin/nothing-runs-this|root".into()],
            Killing::Terminate,
            &shared,
            now(),
        );

        match answer {
            Response::Killed { report } => {
                assert_eq!(report.target, KillTarget::Program);
                assert_eq!(report.done(), 0, "{report:?}");
            }
            other => panic!("it answered {other:?}"),
        }
        let raised = shared.with(|state| state.take_what_the_console_raised());
        assert_eq!(raised.len(), 1);
        assert_eq!(raised[0].kind.as_str(), "agent.process.kill_refused");
    }
}
