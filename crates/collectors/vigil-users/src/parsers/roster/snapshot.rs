use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::reading::AccountsReading;
use crate::parsers::group::privilege_of;
use crate::parsers::sudoers::SudoGrant;

pub const SOURCE: &str = "users";

pub fn accounts_snapshot(taken_at: &str, reading: &AccountsReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());
    let members = members_by_group(reading);

    add_accounts(&mut snapshot, reading, &members);
    add_groups(&mut snapshot, reading, &members);
    add_sudoers(&mut snapshot, reading);
    add_keys(&mut snapshot, reading);
    add_sessions(&mut snapshot, reading);
    add_session_sources(&mut snapshot, reading);

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

type Members<'a> = BTreeMap<&'a str, BTreeSet<&'a str>>;

fn members_by_group<'a>(reading: &AccountsReading<'a>) -> Members<'a> {
    let mut by_primary_gid: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    for entry in reading.passwd {
        by_primary_gid
            .entry(entry.gid)
            .or_default()
            .push(&entry.name);
    }

    let mut members: Members<'a> = BTreeMap::new();
    for group in reading.groups {
        let mut named: BTreeSet<&str> = group.members.iter().map(String::as_str).collect();
        named.extend(
            by_primary_gid
                .get(&group.gid)
                .into_iter()
                .flatten()
                .copied(),
        );
        members.insert(&group.name, named);
    }
    members
}

fn add_accounts(snapshot: &mut Snapshot, reading: &AccountsReading<'_>, members: &Members<'_>) {
    let mut groups_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (group, named) in members {
        for member in named {
            groups_of.entry(member).or_default().push(group);
        }
    }

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
                "groups": groups_of.get(entry.name.as_str()).map(Vec::as_slice).unwrap_or_default(),
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

fn add_groups(snapshot: &mut Snapshot, reading: &AccountsReading<'_>, members: &Members<'_>) {
    let nobody = BTreeSet::new();
    for group in reading.groups {
        let members = members.get(group.name.as_str()).unwrap_or(&nobody);

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
    for session in reading.sessions {
        snapshot.items.insert(
            session.key(),
            json!({
                "user": session.who(),
                "uid": session.uid,
                "line": session.line,
                "from": session.from,
                "remote": session.is_remote(),
                "pid": session.pid,
                "session_id": session.id,
                "service": session.service,
                "type": session.kind,
                "class": session.class,
                "state": session.state,
                "attended": session.attended(),
                "seen_by": session.seen_by(),
            }),
        );
    }
}

fn add_session_sources(snapshot: &mut Snapshot, reading: &AccountsReading<'_>) {
    for source in reading.session_sources {
        snapshot.items.insert(
            format!("session-source|{}", source.name),
            json!({
                "source": source.name,
                "path": source.path,
                "present": source.present,
                "read": source.read,
                "answers": source.answers(),
                "sessions": source.sessions,
                "reason": source.reason,
            }),
        );
    }
}
