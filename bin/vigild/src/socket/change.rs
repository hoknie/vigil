use vigil_model::{AccountChange, ProtocolError, Response, Rfc3339};

use super::Shared;
use crate::accounts::{READING, carry_out, findings};
use crate::helpers::uuid7;

const OFF: &str = "This agent changes no account on this host. Changing accounts from the console \
                   is off until accounts.from_the_console in the users block says otherwise \
                   (collectors/users.yaml, or vigil.yaml on a host of the former layout), and the \
                   daemon reads that key once, at start-up.";

pub fn change(changes: &[AccountChange], shared: &Shared, now: Rfc3339) -> Response {
    if !shared.with(|state| state.accounts_from_the_console()) {
        return refused(ProtocolError::NOT_ALLOWED, OFF);
    }
    if changes.is_empty() {
        return refused(
            ProtocolError::NOTHING_TO_ACT_ON,
            "no change to an account was named",
        );
    }

    let reading = shared.with(|state| state.snapshot(READING).cloned());
    let carried = carry_out(changes, reading.as_ref(), now);
    let raised = findings(&carried, &mut uuid7::mint);
    shared.with(|state| {
        state.record_what_the_console_did(&raised);
        state.ask_for_a_reading(READING);
    });

    for (one, asked) in carried.report.changed.iter().zip(&carried.asked) {
        eprintln!("  console asked to {asked} — {}", one.said);
    }

    Response::Changed {
        report: Box::new(carried.report),
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

    fn root() -> Vec<AccountChange> {
        vec![AccountChange::DeleteUser {
            name: "root".into(),
        }]
    }

    #[test]
    fn an_agent_nobody_switched_this_on_for_refuses_and_names_the_file_that_would() {
        for state in [fixture::state(), fixture::state_that_may_kill()] {
            let shared = Shared::new(state);

            match change(&root(), &shared, now()) {
                Response::Error { error } => {
                    assert_eq!(error.code, ProtocolError::NOT_ALLOWED);
                    assert!(
                        error.message.contains("accounts.from_the_console")
                            && error.message.contains("collectors/users.yaml"),
                        "the refusal names the key and the file it lives in: {error}"
                    );
                }
                other => panic!(
                    "it answered {other:?}: a host where programs may be stopped has not \
                     agreed that its accounts may be changed"
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
                "an agent that changed nothing has nothing new to read"
            );
        }
    }

    #[test]
    fn every_change_answered_asks_for_the_accounts_to_be_read_again_even_one_that_was_refused() {
        let mut state = fixture::state_that_may_change();
        state.record_reading(fixture::reading_of("users", vigil_users::fixture::users()));
        let shared = Shared::new(state);

        change(&root(), &shared, now());

        assert_eq!(
            shared.with(|state| state.take_readings_asked_for()),
            vec![READING.to_string()],
            "a change that stopped half-way has changed the host too, and reading the accounts \
             costs a fraction of a millisecond, so the reading is asked for whatever the outcome"
        );
    }

    #[test]
    fn an_ask_with_no_change_in_it_is_refused_rather_than_answered_with_an_empty_report() {
        let shared = Shared::new(fixture::state_that_may_change());

        match change(&[], &shared, now()) {
            Response::Error { error } => assert_eq!(error.code, ProtocolError::NOTHING_TO_ACT_ON),
            other => panic!("it answered {other:?}"),
        }
    }

    #[test]
    fn a_refusal_is_in_the_agent_s_own_findings_before_the_console_is_answered() {
        let mut state = fixture::state_that_may_change();
        state.record_reading(fixture::reading_of("users", vigil_users::fixture::users()));
        let shared = Shared::new(state);

        let answer = change(&root(), &shared, now());

        match answer {
            Response::Changed { report } => {
                assert_eq!(report.done(), 0);
                assert!(report.changed[0].said.contains("uid 0"), "{report:?}");
            }
            other => panic!("it answered {other:?}"),
        }
        let raised = shared.with(|state| state.take_what_the_console_raised());
        assert_eq!(raised.len(), 1);
        assert_eq!(raised[0].kind.as_str(), "agent.account.change_refused");
        assert_eq!(raised[0].finding_key, "agent.account.change|account|root");
    }
}
