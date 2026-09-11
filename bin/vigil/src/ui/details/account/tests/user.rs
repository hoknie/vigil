use serde_json::json;
use vigil_model::Snapshot;

use super::harness::drawn;
use crate::ui::{Reading, fixture};

#[test]
fn an_accounts_detail_answers_who_this_is_and_what_they_can_do() {
    let page = drawn(&fixture::view(), "account|deploy", 80);

    assert!(page.contains("DEPLOY"), "{page}");
    assert!(page.contains("1000"), "the uid: {page}");
    assert!(page.contains("/bin/bash"), "the shell: {page}");
    assert!(page.contains("locked"), "the password state: {page}");
    assert!(page.contains("root by"), "{page}");
    assert!(page.contains("docker"), "the privileged group: {page}");
    assert!(
        page.contains("may start a container"),
        "and why that group is one: {page}"
    );
    assert!(
        page.contains("(through the group)"),
        "the sudo that reaches it through %wheel: {page}"
    );
    assert!(
        page.contains("/etc/sudoers"),
        "and the file it is in: {page}"
    );
    assert!(
        page.contains("SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ"),
        "its key, whole: {page}"
    );
    assert!(page.contains("pts/0"), "and its session: {page}");
}

#[test]
fn a_shadow_the_agent_could_not_read_is_a_refusal_and_not_an_account_without_a_password() {
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

    let page = drawn(&view, "account|opaque", 80);
    let unbroken = page.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(unbroken.contains("was not readable"), "{page}");
    assert!(
        unbroken.contains("not the same as an account with no password"),
        "{page}"
    );
}

#[test]
fn a_home_the_agent_was_refused_is_named_on_the_accounts_own_detail() {
    let mut view = fixture::view();
    view.readings.put(
        "users",
        Reading::Taken(
            Snapshot::new("users", "2026-09-09T09:00:00.000Z")
                .with(
                    "account|backup",
                    json!({
                        "name": "backup", "uid": 1001, "gid": 1001, "home": "/home/backup",
                        "shell": "/bin/sh", "interactive": true, "password": "locked",
                        "password_permits_login": false, "shadow_readable": true,
                    }),
                )
                .with(
                    "sshkey|backup|unreadable",
                    json!({
                        "user": "backup", "uid": 1001,
                        "source": "/home/backup/.ssh/authorized_keys", "readable": false,
                    }),
                ),
        ),
    );

    let page = drawn(&view, "account|backup", 80);
    let unbroken = page.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(unbroken.contains("was not readable"), "{page}");
    assert!(
        unbroken.contains("may have keys that were never read"),
        "{page}"
    );
}

#[test]
fn a_host_that_keeps_no_login_database_is_not_an_account_with_nobody_on_it() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut()
        && let Some(users) = status
            .agent
            .collectors
            .iter_mut()
            .find(|collector| collector.name == "users")
    {
        users.state = vigil_model::CollectorState::Degraded;
        users.reason = Some("logins are not recorded in a form this build reads".into());
    }

    let page = drawn(&view, "account|backdoor", 80);

    assert!(page.contains("No login recorded"), "{page}");
    assert!(page.contains("incomplete"), "{page}");
}
