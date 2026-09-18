use serde_json::json;

use crate::HIDDEN;
use crate::fixture::{self, row};
use crate::types::{Engine, Subject};

const NEVER_WRITTEN_DOWN: [&str; 2] = ["s.9a7f6e5d4c3b", "s.0011223344556677"];

#[test]
fn a_label_holding_a_token_is_hidden_before_the_reading_is_built_and_the_row_says_it_was() {
    let reading = fixture::engines();

    let web = row(
        &reading,
        Engine::Docker,
        Subject::Container,
        fixture::CONTAINER_MOUNTING_ETC,
    );
    let shell = row(&reading, Engine::Podman, Subject::Container, "tools-shell");

    assert_eq!(web["labels"]["com.shop.api-token"], json!(HIDDEN));
    assert_eq!(web["labels_redacted"], json!(true));
    assert_eq!(shell["labels"]["com.corp.vault-token"], json!(HIDDEN));
    assert_eq!(shell["labels_redacted"], json!(true));
    assert_eq!(
        shell["labels"]["role"],
        json!("shell"),
        "a label that holds no secret keeps its value, or every label reads as hidden and \
         the word stops meaning anything"
    );
}

#[test]
fn nothing_a_secret_was_written_with_reaches_the_reading_anywhere_in_it() {
    let written = serde_json::to_string(&fixture::engines()).expect("a reading is plain data");

    for leaked in NEVER_WRITTEN_DOWN {
        assert!(
            !written.contains(leaked),
            "{leaked} is in the reading, and what reached this buffer has already been \
             written to disk and sent to whatever receiver this host names"
        );
    }
}

#[test]
fn a_secret_of_the_engine_is_a_name_and_a_time_and_the_row_says_the_value_is_not_there() {
    let secret = row(
        &fixture::engines(),
        Engine::Podman,
        Subject::Secret,
        fixture::SECRET,
    );

    assert_eq!(secret["name"], json!(fixture::SECRET));
    assert_eq!(secret["driver"], json!("file"));
    assert_eq!(secret["created_at"], json!("2026-09-02T06:28:00Z"));
    assert_eq!(
        secret["value_redacted"],
        json!(true),
        "the value of a secret is never asked for and never printed, and the row says so out \
         loud: a reader seeing no value must know it was withheld rather than empty"
    );
    assert!(!secret.to_string().contains("value\":"), "{secret}");
}

#[test]
fn the_command_a_container_was_started_with_is_nowhere_in_the_reading() {
    let written = serde_json::to_string(&fixture::engines()).expect("a reading is plain data");

    for said in ["nginx -g", "/srv/api serve", "--collect"] {
        assert!(
            !written.contains(said),
            "{said} is a command line, and a command line is where `mysql -p…` and \
             `curl -H 'Authorization: …'` live. This reading never carries one"
        );
    }
}

#[test]
fn no_environment_variable_of_any_container_is_in_the_reading_because_none_is_ever_asked_for() {
    for engine in Engine::ALL {
        for asked in engine.asks() {
            assert!(
                !asked.arguments.contains(&"inspect"),
                "{:?} would print the environment of a container, which is where a database \
                 password lives on most hosts. Nothing here asks for it",
                asked.arguments
            );
        }
    }
    assert!(
        !serde_json::to_string(&fixture::engines())
            .expect("a reading is plain data")
            .contains("\"Env\""),
        "an environment block reached the reading from somewhere other than `inspect`"
    );
}
