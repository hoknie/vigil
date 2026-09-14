use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::super::key_file::UserKeyFile;
use super::super::reading::AccountsReading;
use super::super::snapshot::accounts_snapshot;
use super::harness::{GROUP, PASSWD, reading_parts, snapshot_of};
use crate::parsers::group::parse_group;
use crate::parsers::sessions::{Session, SessionSource, UTMP};
use crate::parsers::sudoers::parse_sudoers;
use vigil_collect::parse_passwd_entries;

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
            sessions: &[],
            session_sources: &[],
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

fn groups_walked<'a>(snapshot: &'a Snapshot, name: &str) -> Vec<&'a str> {
    snapshot
        .items
        .iter()
        .filter(|(key, _)| key.starts_with("group|"))
        .filter(|(_, group)| {
            group["members"]
                .as_array()
                .is_some_and(|members| members.iter().any(|member| member.as_str() == Some(name)))
        })
        .filter_map(|(_, group)| group["name"].as_str())
        .collect()
}

#[test]
fn every_account_lists_the_groups_whose_member_lists_name_it_its_own_primary_group_included() {
    let mut passwd = parse_passwd_entries(PASSWD);
    passwd[2].gid = 998;
    let groups = parse_group(
        "root:x:0:\nsudo:x:27:deploy,www-data\ndocker:x:998:deploy\ndeploy:x:1000:\nempty:x:5000:\n",
    );
    let built = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: None,
            sudo: &[],
            keys: &[],
            sessions: &[],
            session_sources: &[],
        },
    );

    for snapshot in [&built, &crate::fixture::users()] {
        for (key, account) in snapshot
            .items
            .iter()
            .filter(|(key, _)| key.starts_with("account|"))
        {
            let name = account["name"].as_str().expect("an account has a name");
            let listed: Vec<&str> = account["groups"]
                .as_array()
                .expect("every account item carries the list of its groups")
                .iter()
                .filter_map(Value::as_str)
                .collect();

            assert_eq!(
                listed,
                groups_walked(snapshot, name),
                "{key}: the console trusts this list instead of walking every group, so it must \
                 be exactly the groups whose items name the account, in the order of their names"
            );
        }
    }
    assert_eq!(
        built.items["account|deploy"]["groups"],
        json!(["docker", "sudo"]),
        "a group named on its line and also the account's primary group is listed once"
    );
    assert_eq!(
        built.items["account|www-data"]["groups"],
        json!(["sudo"]),
        "a primary gid no group carries adds nothing, and the list is still there"
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
            sessions: &[],
            session_sources: &[],
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
        remote: true,
        pid: 4021,
        sources: std::collections::BTreeSet::from([UTMP]),
        ..Session::default()
    }];
    let sources = [SessionSource::read(UTMP, "/run/utmp", 1)];
    let build = |taken_at: &str| {
        accounts_snapshot(
            taken_at,
            &AccountsReading {
                passwd: &parts.passwd,
                groups: &parts.groups,
                shadow: Some(&parts.shadow),
                sudo: &parts.sudo,
                keys: &parts.keys,
                sessions: &sessions,
                session_sources: &sources,
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
            sessions: &[],
            session_sources: &[],
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
