use vigil_model::{
    AccountChange, AccountObject, ChangeReport, Changed, Changing, ProtocolError, Response,
};

use super::answered::answered;
use super::asked::Asked;
use super::sentence::{not_offered, sentence};
use super::sheet::{headline, said};

fn deleted(name: &str) -> AccountChange {
    AccountChange::DeleteUser { name: name.into() }
}

fn report(changed: Vec<Changed>) -> ChangeReport {
    ChangeReport {
        acted_at: "2026-09-14T10:00:00.000Z".into(),
        changed,
    }
}

#[test]
fn the_sheet_counts_what_was_done_names_what_was_left_and_says_when_the_list_catches_up() {
    let report = report(vec![
        Changed::done(&deleted("contractor"), "userdel finished"),
        Changed::refused(&deleted("root"), "uid 0 is not deleted"),
    ]);
    let left = vec![(
        "session-source|logind".to_string(),
        "a record of where logins are read from is not a session".to_string(),
    )];

    assert_eq!(headline(&report, &left), "1 OF 3 CHANGE(S)");
    let page = said(&report, &left).join("\n");
    assert!(
        page.contains("[done] account|contractor — userdel finished"),
        "{page}"
    );
    assert!(
        page.contains("[left] account|root — uid 0 is not deleted"),
        "{page}"
    );
    assert!(page.contains("session-source|logind"), "{page}");
    assert!(page.contains("2 of them were not done"), "{page}");
}

#[test]
fn a_sheet_where_everything_was_done_says_the_accounts_are_read_again_straight_away() {
    let report = report(vec![Changed::done(&deleted("contractor"), "done")]);

    let page = said(&report, &[]).join("\n");

    assert!(page.contains("again straight after a change"), "{page}");
    assert!(
        !page.contains("every few minutes"),
        "the list no longer waits for the period of the accounts, and a sheet that says it \
         does sends a person away for five minutes for nothing: {page}"
    );
}

#[test]
fn an_agent_that_has_it_switched_off_is_answered_with_the_key_that_switches_it_on() {
    let asked = answered(Some(Response::Error {
        error: ProtocolError::new(ProtocolError::NOT_ALLOWED, "changing accounts is off"),
    }));

    match asked {
        Asked::Refused { message, advice } => {
            assert_eq!(message, "changing accounts is off");
            assert!(
                advice
                    .expect("advice")
                    .contains("accounts.from_the_console"),
                "a refusal that does not name the key sends a reader looking through the file"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_agent_older_than_the_verb_is_said_to_be_older_and_not_broken() {
    match answered(Some(Response::Error {
        error: ProtocolError::new(ProtocolError::UNKNOWN_QUERY, "unknown query \"change\""),
    })) {
        Asked::Refused { advice, .. } => {
            assert!(advice.expect("advice").contains("older"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_connection_closed_without_an_answer_is_trouble_in_words() {
    assert!(matches!(answered(None), Asked::Trouble(said) if said.contains("without an answer")));
}

#[test]
fn a_change_a_list_does_not_offer_is_refused_in_a_sentence_naming_the_object() {
    assert_eq!(
        not_offered(AccountObject::User, Changing::Create),
        "Accounts are not created from this console."
    );
}

#[test]
fn what_the_daemon_or_a_pane_says_is_turned_into_a_sentence_once() {
    assert_eq!(sentence("the row is gone"), "The row is gone.");
    assert_eq!(sentence("Already said."), "Already said.");
    assert_eq!(sentence(""), "");
}
