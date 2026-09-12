use serde_json::json;
use vigil_model::Change;

use crate::rules::fixture;
use crate::rules::verdict::accounts;

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
