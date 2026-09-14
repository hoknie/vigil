use serde_json::json;
use vigil_model::Change;
use vigil_rules::{diff, findings_for};

use super::set::account_rules;
use super::verdict::accounts;
use crate::fixture;

#[test]
fn the_first_reading_after_the_agent_starts_listing_the_groups_of_every_account_raises_nothing() {
    let upgraded = fixture::users();
    let mut older = fixture::users();
    for (key, item) in older.items.iter_mut() {
        if key.starts_with("account|") {
            item.as_object_mut()
                .expect("an account item is an object")
                .remove("groups");
        }
    }

    let changes = diff(&older, &upgraded);

    assert!(
        !changes.is_empty()
            && changes.iter().all(|change| matches!(
                change,
                Change::Changed { key, .. } if key.starts_with("account|")
            )),
        "the upgrade changes every account item and nothing else: {changes:?}"
    );
    assert_eq!(
        findings_for(account_rules(), &changes),
        Vec::<(String, String)>::new(),
        "every account item changes once after an upgrade, and an account rule that read the \
         whole item instead of its own fields would alert on every account of every host"
    );
}

#[test]
fn joining_a_privileged_group_changes_the_account_too_and_only_the_group_speaks_about_it() {
    let after = fixture::users();
    let mut before = fixture::users();
    for (key, field, gone) in [
        ("group|wheel", "members", "deploy"),
        ("account|deploy", "groups", "wheel"),
    ] {
        before
            .items
            .get_mut(key)
            .and_then(|item| item[field].as_array_mut())
            .expect("the sample has deploy in wheel")
            .retain(|name| *name != gone);
    }

    let changes = diff(&before, &after);
    let changed: Vec<&str> = changes
        .iter()
        .map(|change| match change {
            Change::Added { key, .. }
            | Change::Changed { key, .. }
            | Change::Removed { key, .. } => key.as_str(),
        })
        .collect();

    assert_eq!(changed, vec!["account|deploy", "group|wheel"]);
    assert_eq!(
        findings_for(account_rules(), &changes),
        vec![(
            "privileged_group_member_added".to_string(),
            "user.group.privileged_member_added".to_string()
        )],
        "the account now carries its groups, so one usermod changes two items, and a person \
         joining a group is still one finding"
    );
}

#[test]
fn exactly_one_account_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "useradd -u 0 toor",
            Change::Added {
                key: "account|toor".into(),
                after: fixture::account("toor", 0, "/bin/bash"),
            },
            "second_root_account",
            "user.account.uid0",
        ),
        (
            "usermod -u 0 backup",
            Change::Changed {
                key: "account|backup".into(),
                before: fixture::account("backup", 34, "/bin/sh"),
                after: fixture::account("backup", 0, "/bin/sh"),
            },
            "second_root_account",
            "user.account.uid0",
        ),
        (
            "useradd deploy",
            Change::Added {
                key: "account|deploy".into(),
                after: fixture::account("deploy", 1000, "/bin/bash"),
            },
            "new_account",
            "user.account.new",
        ),
        (
            "userdel deploy",
            Change::Removed {
                key: "account|deploy".into(),
                before: fixture::account("deploy", 1000, "/bin/bash"),
            },
            "removed_account",
            "user.account.removed",
        ),
        (
            "passwd -u backup (the lock comes off and the date moves)",
            Change::Changed {
                key: "account|backup".into(),
                before: fixture::account_with_password("backup", 34, "locked", 18000),
                after: fixture::account_with_password("backup", 34, "set", 19200),
            },
            "account_unlocked",
            "user.account.unlocked",
        ),
        (
            "passwd deploy",
            Change::Changed {
                key: "account|deploy".into(),
                before: fixture::account_with_password("deploy", 1000, "set", 19000),
                after: fixture::account_with_password("deploy", 1000, "set", 19200),
            },
            "password_changed",
            "user.password.changed",
        ),
        (
            "usermod -aG sudo deploy",
            Change::Changed {
                key: "group|sudo".into(),
                before: fixture::group("sudo", &["alice"], Some("may run commands as any user")),
                after: fixture::group(
                    "sudo",
                    &["alice", "deploy"],
                    Some("may run commands as any user"),
                ),
            },
            "privileged_group_member_added",
            "user.group.privileged_member_added",
        ),
        (
            "echo 'deploy ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/deploy",
            Change::Added {
                key: "sudoer|deploy".into(),
                after: fixture::sudoer("deploy", "ALL=(ALL) NOPASSWD: ALL", true, true),
            },
            "sudo_grant_added",
            "user.group.privileged_member_added",
        ),
        (
            "cat key >> ~deploy/.ssh/authorized_keys",
            Change::Added {
                key: "sshkey|deploy|SHA256:abc".into(),
                after: fixture::ssh_key("deploy", 1000, "SHA256:abc", None, None),
            },
            "ssh_key_added",
            "user.sshkey.added",
        ),
        (
            "the same key removed again",
            Change::Removed {
                key: "sshkey|deploy|SHA256:abc".into(),
                before: fixture::ssh_key("deploy", 1000, "SHA256:abc", None, None),
            },
            "ssh_key_removed",
            "user.sshkey.removed",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = accounts(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_login_is_carried_in_the_snapshot_and_no_rule_speaks_about_it_yet() {
    let change = Change::Added {
        key: "session|deploy|pts/0".into(),
        after: json!({
            "user": "deploy", "line": "pts/0", "from": "10.0.0.7",
            "remote": true, "pid": 4021,
        }),
    };

    assert!(accounts(&change).is_empty());
}
