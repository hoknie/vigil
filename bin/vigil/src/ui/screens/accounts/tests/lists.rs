use serde_json::json;
use vigil_model::Snapshot;

use super::harness::drawn_at;
use crate::ui::screens::accounts::{keys, rows};
use crate::ui::{Reading, Search, Subject, fixture};

#[test]
fn nothing_in_the_reading_is_on_no_list_at_all() {
    let mut view = fixture::view();
    let Reading::Taken(snapshot) = view.reading("users") else {
        panic!("the fixture has a reading");
    };
    view.readings.put(
        "users",
        Reading::Taken(
            snapshot
                .clone()
                .with("keyring|root|0x1234", json!({"from": "a newer agent"})),
        ),
    );

    let Reading::Taken(snapshot) = view.reading("users") else {
        unreachable!()
    };
    let mut reachable: Vec<String> = Subject::on(&view)
        .into_iter()
        .flat_map(|subject| keys(&view, subject, &Search::default()))
        .collect();
    reachable.sort();
    reachable.dedup();

    let mut held: Vec<String> = snapshot.items.keys().cloned().collect();
    held.sort();

    assert_eq!(held, reachable, "an object of the reading is on no list");
}

#[test]
fn a_refusal_is_at_the_top_of_its_list_rather_than_buried_under_forty_rows() {
    let view = fixture::view();

    let refusals = keys(&view, Subject::Keys, &Search::default());
    assert!(refusals[0].starts_with("sshkey|backup"), "{refusals:?}");

    let ssh = keys(&view, Subject::SshUsers, &Search::default());
    assert!(
        ssh[0].starts_with("sshkey|backup") || ssh[0] == "account|backup",
        "{ssh:?}"
    );
}

#[test]
fn an_ssh_user_is_an_account_by_its_own_key_so_a_suppression_can_be_written_from_it() {
    let view = fixture::view();
    let rows = rows(&view, Subject::SshUsers, &Search::default());

    let names: Vec<String> = rows.iter().map(|row| row.key.clone()).collect();
    assert!(names.contains(&"account|deploy".to_string()), "{names:?}");
    assert!(
        names.iter().any(|key| key.starts_with("sshkey|backup")),
        "the key file of a name that is not an account is still a row: {names:?}"
    );
    assert!(
        drawn_at(&view, Subject::SshUsers, 120).contains("not in /etc/passwd"),
        "and the row says which of the two it is"
    );
}

#[test]
fn an_object_of_a_kind_this_console_does_not_know_gets_a_list_of_its_own() {
    let mut view = fixture::view();
    view.readings.put(
        "users",
        Reading::Taken(
            Snapshot::new("users", "2026-09-09T09:00:00.000Z")
                .with(
                    "account|root",
                    json!({"name": "root", "uid": 0, "shell": "/bin/sh", "shadow_readable": true}),
                )
                .with("keyring|root|0x1234", json!({"from": "a newer agent"})),
        ),
    );

    let page = drawn_at(&view, Subject::Other, 120);

    assert!(page.contains("other"), "the list is on the submenu: {page}");
    assert!(page.contains("keyring|root|0x1234"), "{page}");
    assert!(page.contains("does not know"), "{page}");
}
