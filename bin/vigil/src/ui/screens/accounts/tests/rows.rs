use serde_json::json;
use vigil_model::Snapshot;

use super::harness::drawn_at;
use crate::ui::{Reading, Subject, fixture};

#[test]
fn an_account_says_its_password_state_its_shell_and_how_it_reaches_root() {
    let page = drawn_at(&fixture::view(), Subject::Users, 200);

    assert!(page.contains("backdoor"), "{page}");
    assert!(page.contains("uid 0"), "{page}");
    assert!(
        page.contains("locked"),
        "the password state is a word, not a colour: {page}"
    );
    assert!(page.contains("docker"), "{page}");
    assert!(page.contains("sudo to every command"), "{page}");
}

#[test]
fn a_shadow_line_the_agent_could_not_read_says_unknown_rather_than_nothing() {
    let mut view = fixture::view();
    view.readings.put(
        "users",
        Reading::Taken(Snapshot::new("users", "2026-09-09T09:00:00.000Z").with(
            "account|opaque",
            json!({
                "name": "opaque", "uid": 1001, "gid": 1001, "home": "/home/opaque",
                "shell": "/bin/sh", "interactive": true, "password": null,
                "password_permits_login": null, "shadow_readable": false,
            }),
        )),
    );

    let page = drawn_at(&view, Subject::Users, 200);
    assert!(
        page.contains("unknown"),
        "an empty cell would read as an account with no password: {page}"
    );
    assert!(
        page.contains("password state could not be read"),
        "and the tally counts it, on the list it belongs to: {page}"
    );
}

#[test]
fn a_group_that_is_root_by_a_longer_road_says_why_on_its_row() {
    let page = drawn_at(&fixture::view(), Subject::Groups, 120);

    assert!(page.contains("may start a container"), "{page}");
    assert!(page.contains("that grant power"), "{page}");
}

#[test]
fn a_grant_to_a_group_says_which_accounts_it_reaches() {
    let page = drawn_at(&fixture::view(), Subject::Sudo, 120);

    assert!(page.contains("%wheel"), "{page}");
    assert!(page.contains("deploy"), "{page}");
    assert!(page.contains("every command"), "{page}");
}

#[test]
fn a_home_the_agent_was_refused_is_a_row_of_the_keys_and_of_the_ssh_users() {
    let keys = drawn_at(&fixture::view(), Subject::Keys, 120);
    assert!(keys.contains("was not readable"), "{keys}");
    assert!(keys.contains("backup"), "{keys}");
    assert!(keys.contains("key file(s) the agent was refused"), "{keys}");

    let users = drawn_at(&fixture::view(), Subject::SshUsers, 120);
    assert!(users.contains("backup"), "{users}");
    assert!(users.contains("refused"), "{users}");
}
