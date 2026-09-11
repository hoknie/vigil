use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::reading::AccountsReading;
use crate::parsers::accounts::group::privilege_of;
use crate::parsers::accounts::sudoers::SudoGrant;

pub const SOURCE: &str = "users";

pub fn accounts_snapshot(taken_at: &str, reading: &AccountsReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_accounts(&mut snapshot, reading);
    add_groups(&mut snapshot, reading);
    add_sudoers(&mut snapshot, reading);
    add_keys(&mut snapshot, reading);
    add_sessions(&mut snapshot, reading);

    snapshot
}

const NON_INTERACTIVE_SHELLS: &[&str] = &[
    "/usr/sbin/nologin",
    "/sbin/nologin",
    "/usr/bin/nologin",
    "/bin/false",
    "/usr/bin/false",
    "",
];

fn add_accounts(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    for entry in reading.passwd {
        let facts = reading
            .shadow
            .and_then(|shadow| shadow.get(&entry.name).copied());

        snapshot.items.insert(
            format!("account|{}", entry.name),
            json!({
                "name": entry.name,
                "uid": entry.uid,
                "gid": entry.gid,
                "home": entry.home,
                "shell": entry.shell,
                "interactive": !NON_INTERACTIVE_SHELLS.contains(&entry.shell.as_str()),
                "password": facts.map(|facts| facts.password.as_str()),
                "password_permits_login": facts.map(|facts| facts.password.permits_login()),
                "password_last_change_day": facts.and_then(|facts| facts.last_change_day),
                "password_max_age_days": facts.and_then(|facts| facts.max_age_days),
                "account_expires_day": facts.and_then(|facts| facts.expires_day),
                "shadow_readable": facts.is_some(),
            }),
        );
    }
}

fn add_groups(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    for group in reading.groups {
        let mut members: BTreeSet<&str> = group.members.iter().map(String::as_str).collect();
        for entry in reading.passwd {
            if entry.gid == group.gid {
                members.insert(&entry.name);
            }
        }

        let privilege = privilege_of(&group.name);
        snapshot.items.insert(
            format!("group|{}", group.name),
            json!({
                "name": group.name,
                "gid": group.gid,
                "members": members.iter().collect::<Vec<_>>(),
                "privileged": privilege.is_some(),
                "privilege": privilege,
            }),
        );
    }
}

fn add_sudoers(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    let mut by_principal: BTreeMap<&str, Vec<&SudoGrant>> = BTreeMap::new();
    for grant in reading.sudo {
        by_principal.entry(&grant.who).or_default().push(grant);
    }

    for (who, grants) in by_principal {
        let specs: Vec<Value> = grants
            .iter()
            .map(|grant| {
                json!({
                    "source": grant.source,
                    "spec": grant.spec,
                    "spec_redacted": grant.spec_redacted,
                    "nopasswd": grant.nopasswd,
                    "all_commands": grant.all_commands,
                })
            })
            .collect();

        snapshot.items.insert(
            format!("sudoer|{who}"),
            json!({
                "who": who,
                "group": grants.first().is_some_and(|grant| grant.is_group()),
                "rules": specs,
                "nopasswd": grants.iter().any(|grant| grant.nopasswd),
                "all_commands": grants.iter().any(|grant| grant.all_commands),
                "spec_redacted": grants.iter().any(|grant| grant.spec_redacted),
            }),
        );
    }
}

fn add_keys(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    for file in reading.keys {
        if !file.readable {
            snapshot.items.insert(
                format!("sshkey|{}|unreadable", file.user),
                json!({
                    "user": file.user,
                    "uid": file.uid,
                    "source": file.path,
                    "readable": false,
                }),
            );
            continue;
        }

        for key in &file.keys {
            snapshot.items.insert(
                format!("sshkey|{}|{}", file.user, key.fingerprint),
                json!({
                    "user": file.user,
                    "uid": file.uid,
                    "algorithm": key.algorithm,
                    "fingerprint": key.fingerprint,
                    "comment": key.comment,
                    "options": key.options,
                    "source": file.path,
                    "readable": true,
                }),
            );
        }
    }
}

fn add_sessions(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    let Some(sessions) = reading.sessions else {
        return;
    };

    for session in sessions {
        snapshot.items.insert(
            format!("session|{}|{}", session.user, session.line),
            json!({
                "user": session.user,
                "line": session.line,
                "from": session.from,
                "remote": session.is_remote(),
                "pid": session.pid,
            }),
        );
    }
}
