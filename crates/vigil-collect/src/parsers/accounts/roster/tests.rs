use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::Snapshot;

use super::key_file::UserKeyFile;
use super::reading::AccountsReading;
use super::snapshot::accounts_snapshot;
use crate::parsers::accounts::authorized_keys::parse_authorized_keys;
use crate::parsers::accounts::group::{GroupEntry, parse_group};
use crate::parsers::accounts::passwd::{PasswdEntry, parse_passwd_entries};
use crate::parsers::accounts::shadow::{ShadowFacts, parse_shadow};
use crate::parsers::accounts::sudoers::{SudoGrant, parse_sudoers};
use crate::parsers::accounts::utmp::Session;

const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
deploy:x:1000:1000:deploy:/home/deploy:/bin/bash
";
const GROUP: &str = "\
root:x:0:
sudo:x:27:deploy
docker:x:998:
deploy:x:1000:
";
const SHADOW: &str = "\
root:$6$salt$hash:19000:0:99999:7:::
www-data:*:19000:0:99999:7:::
deploy:!$6$salt$hash:19100:0:99999:7:::
";
const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr deploy@builder";

struct Parts {
    passwd: Vec<PasswdEntry>,
    groups: Vec<GroupEntry>,
    shadow: BTreeMap<String, ShadowFacts>,
    sudo: Vec<SudoGrant>,
    keys: Vec<UserKeyFile>,
}

fn reading_parts() -> Parts {
    Parts {
        passwd: parse_passwd_entries(PASSWD),
        groups: parse_group(GROUP),
        shadow: parse_shadow(SHADOW),
        sudo: parse_sudoers("deploy ALL=(ALL) NOPASSWD: ALL\n", "/etc/sudoers.d/deploy").grants,
        keys: vec![UserKeyFile {
            user: "deploy".into(),
            uid: 1000,
            path: "/home/deploy/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        }],
    }
}

fn snapshot_of(shadow: Option<&BTreeMap<String, ShadowFacts>>) -> Snapshot {
    let parts = reading_parts();
    accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &parts.passwd,
            groups: &parts.groups,
            shadow,
            sudo: &parts.sudo,
            keys: &parts.keys,
            sessions: None,
        },
    )
}

#[test]
fn keys_every_object_in_a_form_a_person_can_copy_into_a_suppression() {
    let parts = reading_parts();
    let snapshot = snapshot_of(Some(&parts.shadow));

    assert_eq!(snapshot.source, "users");
    for expected in [
        "account|root",
        "account|deploy",
        "group|sudo",
        "group|docker",
        "sudoer|deploy",
        "sshkey|deploy|SHA256:ie96zLdp9BkGxjqBG3AldJ2imVMNe8sXmmyorODpGCM",
    ] {
        assert!(
            snapshot.items.contains_key(expected),
            "missing {expected}: {:?}",
            snapshot.items.keys().collect::<Vec<_>>()
        );
    }
}

#[test]
fn an_unreadable_shadow_is_marked_unreadable_rather_than_left_looking_unlocked() {
    let parts = reading_parts();
    let with = snapshot_of(Some(&parts.shadow));
    let without = snapshot_of(None);

    assert_eq!(with.items["account|deploy"]["password"], "locked");
    assert_eq!(
        with.items["account|deploy"]["password_permits_login"],
        false
    );
    assert_eq!(with.items["account|deploy"]["shadow_readable"], true);
    assert_eq!(
        with.items["account|deploy"]["password_last_change_day"],
        19100
    );

    assert_eq!(without.items["account|deploy"]["password"], Value::Null);
    assert_eq!(
        without.items["account|deploy"]["password_permits_login"],
        Value::Null
    );
    assert_eq!(
        without.items["account|deploy"]["shadow_readable"], false,
        "'we were not allowed to look' and 'no password is set' must not be one document"
    );
}

#[test]
fn nothing_from_the_password_file_reaches_the_snapshot() {
    let parts = reading_parts();
    let snapshot = snapshot_of(Some(&parts.shadow));

    let printed = serde_json::to_string(&snapshot).expect("serialises");
    for secret in ["$6$", "salt", "hash"] {
        assert!(!printed.contains(secret), "{secret} reached the snapshot");
    }
}

#[test]
fn a_primary_group_membership_is_a_membership_like_any_other() {
    let mut passwd = parse_passwd_entries(PASSWD);
    passwd[2].gid = 998;
    let groups = parse_group(GROUP);

    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: None,
            sudo: &[],
            keys: &[],
            sessions: None,
        },
    );

    assert_eq!(snapshot.items["group|docker"]["members"][0], "deploy");
    assert_eq!(snapshot.items["group|docker"]["privileged"], true);
    assert!(
        snapshot.items["group|docker"]["privilege"]
            .as_str()
            .expect("a reason")
            .contains("root"),
        "the reason a person is shown has to say why docker is in this list"
    );
}

#[test]
fn an_unreadable_authorized_keys_file_leaves_a_marker_not_a_silence() {
    let keys = [UserKeyFile {
        user: "alice".into(),
        uid: 1001,
        path: "/home/alice/.ssh/authorized_keys".into(),
        readable: false,
        keys: Vec::new(),
    }];
    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &[],
            groups: &[],
            shadow: None,
            sudo: &[],
            keys: &keys,
            sessions: None,
        },
    );

    assert_eq!(snapshot.items["sshkey|alice|unreadable"]["readable"], false);
}

#[test]
fn the_same_reading_twice_produces_the_same_document() {
    let parts = reading_parts();
    let sessions = [Session {
        user: "deploy".into(),
        line: "pts/0".into(),
        from: "10.0.0.7".into(),
        pid: 4021,
    }];
    let build = |taken_at: &str| {
        accounts_snapshot(
            taken_at,
            &AccountsReading {
                passwd: &parts.passwd,
                groups: &parts.groups,
                shadow: Some(&parts.shadow),
                sudo: &parts.sudo,
                keys: &parts.keys,
                sessions: Some(&sessions),
            },
        )
    };

    assert_eq!(
        build("2026-09-09T12:00:00.000Z").items,
        build("2026-09-09T12:00:30.000Z").items
    );
}

#[test]
fn a_sudoers_grant_is_one_item_per_principal_whatever_the_file_looks_like() {
    let sudo = parse_sudoers(
        "deploy ALL=(ALL) NOPASSWD: ALL\ndeploy ALL=(root) /usr/bin/systemctl\n",
        "/etc/sudoers",
    )
    .grants;
    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &[],
            groups: &[],
            shadow: None,
            sudo: &sudo,
            keys: &[],
            sessions: None,
        },
    );

    assert_eq!(snapshot.items.len(), 1);
    assert_eq!(
        snapshot.items["sudoer|deploy"]["rules"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(snapshot.items["sudoer|deploy"]["nopasswd"], true);
}
