use serde_json::json;
use vigil_model::AccountChange;
use vigil_users::fixture::users;

use super::aim;
use crate::accounts::fields::flag;
use crate::accounts::ours::Ours;

const OTHER: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq ci@build";

fn ours() -> Ours {
    Ours { uid: 998, pid: 77 }
}

fn refused(change: AccountChange) -> String {
    aim(&change, &users(), ours()).expect_err("refused")
}

fn allowed(change: AccountChange) {
    if let Err(why) = aim(&change, &users(), ours()) {
        panic!("{change:?} was refused: {why}");
    }
}

fn deploy_key() -> String {
    users()
        .items
        .keys()
        .find(|key| key.starts_with("sshkey|deploy|"))
        .and_then(|key| key.rsplit('|').next())
        .expect("deploy has a key")
        .to_string()
}

fn update_user(
    name: &str,
    shell: Option<&str>,
    locked: Option<bool>,
    groups: Option<Vec<String>>,
) -> AccountChange {
    AccountChange::UpdateUser {
        name: name.into(),
        shell: shell.map(String::from),
        home: None,
        comment: None,
        locked,
        groups,
    }
}

#[test]
fn the_superuser_is_neither_deleted_nor_locked_whatever_the_console_asks() {
    assert!(
        refused(AccountChange::DeleteUser {
            name: "root".into()
        })
        .contains("uid 0")
    );
    assert!(refused(update_user("root", None, Some(true), None)).contains("uid 0"));
    allowed(update_user("root", Some("/bin/sh"), None, None));
}

#[test]
fn the_account_the_agent_runs_as_is_not_deleted_on_request() {
    assert!(
        refused(AccountChange::DeleteUser {
            name: "svc-runner".into()
        })
        .contains("runs as")
    );
    allowed(AccountChange::DeleteUser {
        name: "contractor".into(),
    });
}

#[test]
fn an_account_or_group_no_longer_in_the_reading_is_refused_by_name() {
    assert!(
        refused(AccountChange::DeleteUser {
            name: "mallory".into()
        })
        .contains("mallory")
    );
    assert!(
        refused(AccountChange::DeleteGroup {
            name: "docker".into()
        })
        .contains("docker")
    );
    assert!(
        refused(AccountChange::CreateGroup {
            name: "wheel".into(),
            members: Vec::new()
        })
        .contains("already")
    );
}

#[test]
fn an_update_that_changes_nothing_is_refused_rather_than_run() {
    assert!(refused(update_user("deploy", None, None, None)).contains("nothing to change"));
}

#[test]
fn what_goes_to_the_account_tools_is_checked_before_any_of_it_is_run() {
    assert!(refused(update_user("deploy", Some("bash"), None, None)).contains("absolute"));
    assert!(
        refused(update_user(
            "deploy",
            None,
            None,
            Some(vec!["docker".into()])
        ))
        .contains("docker")
    );
    assert!(
        refused(AccountChange::CreateGroup {
            name: "-g0".into(),
            members: Vec::new()
        })
        .contains("name")
    );
}

#[test]
fn the_group_of_the_superuser_is_not_deleted() {
    assert!(
        refused(AccountChange::DeleteGroup {
            name: "root".into()
        })
        .contains("gid 0")
    );
}

#[test]
fn a_grant_that_lives_only_in_etc_sudoers_is_left_for_a_person_with_visudo() {
    let why = refused(AccountChange::DeleteSudo {
        who: "%wheel".into(),
    });

    assert!(why.contains("/etc/sudoers"), "{why}");
}

#[test]
fn a_grant_in_sudoers_d_is_rewritten_only_with_rules_and_never_emptied_by_an_update() {
    let mut reading = users();
    reading.items.insert(
        "sudoer|deploy".into(),
        json!({"who": "deploy", "rules": [{"source": "/etc/sudoers.d/deploy", "spec": "ALL=(ALL) ALL"}]}),
    );

    assert!(
        aim(
            &AccountChange::UpdateSudo {
                who: "deploy".into(),
                rules: vec!["ALL=(root) /usr/bin/id".into()]
            },
            &reading,
            ours()
        )
        .is_ok()
    );
    let emptied = aim(
        &AccountChange::UpdateSudo {
            who: "deploy".into(),
            rules: Vec::new(),
        },
        &reading,
        ours(),
    )
    .expect_err("an empty grant is a delete");
    assert!(emptied.contains("delete"), "{emptied}");
}

#[test]
fn a_key_file_the_agent_could_not_read_is_not_changed_blind() {
    let mut reading = users();
    reading.items.insert(
        "account|backup".into(),
        json!({"name": "backup", "uid": 1001, "gid": 1001, "home": "/home/backup"}),
    );

    let why = aim(
        &AccountChange::DeleteSshUser {
            user: "backup".into(),
        },
        &reading,
        ours(),
    )
    .expect_err("refused");

    assert!(why.contains("could not be read"), "{why}");
}

#[test]
fn a_key_already_letting_the_account_in_is_not_added_twice_and_a_new_one_is() {
    let deploy = users().items[&format!("sshkey|deploy|{}", deploy_key())].clone();
    let line = format!(
        "{} {}",
        "ssh-ed25519", "AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr"
    );
    assert!(flag(&deploy, "readable"));

    assert!(
        refused(AccountChange::CreateKey {
            user: "deploy".into(),
            line
        })
        .contains("already")
    );
    allowed(AccountChange::CreateKey {
        user: "deploy".into(),
        line: OTHER.into(),
    });
    assert!(
        refused(AccountChange::CreateSshUser {
            user: "deploy".into(),
            line: OTHER.into()
        })
        .contains("already")
    );
    allowed(AccountChange::CreateSshUser {
        user: "www-data".into(),
        line: OTHER.into(),
    });
}

#[test]
fn a_session_is_ended_by_its_logind_id_or_its_pid_and_never_when_it_is_pid_1() {
    allowed(AccountChange::DeleteSession {
        key: "session|deploy|pts/0".into(),
    });
    assert!(
        refused(AccountChange::DeleteSession {
            key: "session-source|logind".into()
        })
        .contains("not a session")
    );

    let mut reading = users();
    reading.items.insert(
        "session|root|pid:1".into(),
        json!({"user": "root", "pid": 1, "session_id": ""}),
    );
    assert!(
        aim(
            &AccountChange::DeleteSession {
                key: "session|root|pid:1".into()
            },
            &reading,
            ours()
        )
        .expect_err("pid 1")
        .contains("pid 1")
    );
}

fn with_an_account_named(name: &str) -> vigil_model::Snapshot {
    let mut reading = users();
    let mut account = reading.items["account|deploy"].clone();
    account["name"] = json!(name);
    account["uid"] = json!(1_777);
    reading.items.insert(format!("account|{name}"), account);
    reading
}

#[test]
fn an_account_with_a_capital_letter_that_is_on_this_host_can_be_changed_and_deleted() {
    let reading = with_an_account_named("Deploy");

    for change in [
        AccountChange::DeleteUser {
            name: "Deploy".into(),
        },
        update_user("Deploy", Some("/bin/bash"), None, None),
    ] {
        if let Err(why) = aim(&change, &reading, ours()) {
            panic!(
                "the rule for a name this agent writes is a rule for new names, and {change:?} \
                 names an account that already exists: {why}"
            );
        }
    }
}

#[test]
fn a_new_group_is_still_named_the_way_the_account_tools_would_name_it() {
    let why = refused(AccountChange::CreateGroup {
        name: "Ops".into(),
        members: Vec::new(),
    });

    assert!(
        why.contains("is not a group name this agent writes"),
        "{why}"
    );
}

#[test]
fn a_name_with_a_command_in_it_that_is_not_on_this_host_is_refused_as_absent() {
    let why = refused(AccountChange::DeleteUser {
        name: "deploy;id".into(),
    });

    assert!(why.contains("there is no account deploy;id"), "{why}");
}

#[test]
fn a_name_a_tool_would_read_as_an_option_is_refused_before_anything_is_looked_up() {
    let why = refused(AccountChange::DeleteUser { name: "-rf".into() });

    assert!(why.contains("begins with a dash"), "{why}");
}

#[test]
fn a_grant_to_a_sudoers_alias_is_not_changed_from_the_console() {
    let why = refused(AccountChange::DeleteSudo { who: "ALL".into() });

    assert!(why.contains("sudoers alias"), "{why}");
}
