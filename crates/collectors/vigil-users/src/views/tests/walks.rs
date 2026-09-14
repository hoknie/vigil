use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Showing, haystack};

use crate::types::{Kind, Subject};
use crate::views::facts::{attended, could_log_in, privileged, readable, remote};
use crate::views::fields::{members, text};

pub(super) fn every(reading: &Snapshot, kind: Kind) -> Vec<(&str, &Value)> {
    reading
        .items
        .iter()
        .filter(|(key, _)| Kind::of(key) == kind)
        .map(|(key, item)| (key.as_str(), item))
        .collect()
}

pub(super) fn groups_of<'a>(reading: &'a Snapshot, name: &str) -> Vec<(&'a str, &'a Value)> {
    every(reading, Kind::Group)
        .into_iter()
        .filter(|(_, group)| members(group).any(|member| member == name))
        .collect()
}

pub(super) fn sudo_for<'a>(reading: &'a Snapshot, name: &str) -> Vec<(&'a str, &'a Value)> {
    let groups: Vec<&str> = groups_of(reading, name)
        .into_iter()
        .filter_map(|(_, group)| text(group, "name"))
        .collect();
    every(reading, Kind::Sudoer)
        .into_iter()
        .filter(|(_, grant)| match text(grant, "who") {
            Some(who) => match who.strip_prefix('%') {
                Some(group) => groups.contains(&group),
                None => who == name,
            },
            None => false,
        })
        .collect()
}

pub(super) fn route_to_root(reading: &Snapshot, account: &Value) -> String {
    let name = text(account, "name").unwrap_or_default();

    let mut routes: Vec<String> = Vec::new();
    if account.get("uid").and_then(Value::as_u64) == Some(0) {
        routes.push("uid 0".to_string());
    }
    routes.extend(
        groups_of(reading, name)
            .into_iter()
            .filter(|(_, group)| privileged(group))
            .filter_map(|(_, group)| text(group, "name").map(str::to_string)),
    );
    let grants = sudo_for(reading, name);
    if !grants.is_empty() {
        let all = grants
            .iter()
            .any(|(_, grant)| grant.get("all_commands").and_then(Value::as_bool) == Some(true));
        let nopasswd = grants
            .iter()
            .any(|(_, grant)| grant.get("nopasswd").and_then(Value::as_bool) == Some(true));
        routes.push(match (all, nopasswd) {
            (true, true) => "sudo to every command, without a password".to_string(),
            (true, false) => "sudo to every command".to_string(),
            (false, true) => "sudo, without a password".to_string(),
            (false, false) => "sudo".to_string(),
        });
    }
    routes.join(" · ")
}

pub(super) fn keys_of<'a>(reading: &'a Snapshot, user: &str) -> Vec<&'a str> {
    every(reading, Kind::Key)
        .into_iter()
        .filter(|(_, key)| text(key, "user") == Some(user))
        .map(|(key, _)| key)
        .collect()
}

pub(super) fn sessions_of<'a>(reading: &'a Snapshot, user: &str) -> Vec<&'a str> {
    every(reading, Kind::Session)
        .into_iter()
        .filter(|(_, session)| text(session, "user") == Some(user))
        .map(|(key, _)| key)
        .collect()
}

pub(super) fn grants_to_group<'a>(reading: &'a Snapshot, group: &str) -> Vec<&'a Value> {
    every(reading, Kind::Sudoer)
        .into_iter()
        .filter(|(_, grant)| text(grant, "who") == Some(&format!("%{group}")))
        .map(|(_, grant)| grant)
        .collect()
}

pub(super) fn reached_by<'a>(reading: &'a Snapshot, who: &str) -> Vec<&'a str> {
    let named: Vec<&str> = match who.strip_prefix('%') {
        Some(group) => every(reading, Kind::Group)
            .into_iter()
            .filter(|(_, found)| text(found, "name") == Some(group))
            .flat_map(|(_, found)| members(found))
            .collect(),
        None => vec![who],
    };
    every(reading, Kind::Account)
        .into_iter()
        .filter_map(|(_, account)| text(account, "name"))
        .filter(|name| named.contains(name))
        .collect()
}

pub(super) fn rows(reading: &Snapshot, subject: Subject, showing: &Showing<'_>) -> Vec<String> {
    if subject == Subject::SshUsers {
        return ssh_users(reading, showing);
    }
    let kinds = subject.kinds();
    let mut rows: Vec<(u8, String)> = reading
        .items
        .iter()
        .filter(|(key, _)| kinds.contains(&Kind::of(key)))
        .filter(|(key, item)| showing.matches(key, item))
        .map(|(key, item)| (rank(Kind::of(key), item), key.clone()))
        .collect();
    rows.sort();
    rows.into_iter().map(|(_, key)| key).collect()
}

fn rank(kind: Kind, item: &Value) -> u8 {
    match kind {
        Kind::Account => u8::from(!could_log_in(item)),
        Kind::Group => u8::from(!privileged(item)),
        Kind::Key => u8::from(readable(item)),
        Kind::Session => match (attended(item), remote(item)) {
            (true, true) => 0,
            (true, false) => 1,
            (false, _) => 2,
        },
        Kind::SessionSource => 3,
        _ => 0,
    }
}

fn ssh_users(reading: &Snapshot, showing: &Showing<'_>) -> Vec<String> {
    let mut names: Vec<&str> = every(reading, Kind::Key)
        .into_iter()
        .filter_map(|(_, item)| text(item, "user"))
        .collect();
    names.sort_unstable();
    names.dedup();

    let mut rows: Vec<(bool, String, String)> = names
        .into_iter()
        .filter_map(|name| {
            let account = every(reading, Kind::Account)
                .into_iter()
                .find(|(_, item)| text(item, "name") == Some(name))
                .map(|(key, _)| key.to_string());
            let key =
                account.or_else(|| keys_of(reading, name).first().map(|key| key.to_string()))?;
            Some((name.to_string(), key))
        })
        .filter(|(name, key)| {
            let mut searched = haystack(key, &reading.items[key]);
            for other in keys_of(reading, name) {
                searched.push(' ');
                searched.push_str(&haystack(other, &reading.items[other]));
            }
            showing.search.is_empty()
                || searched
                    .to_lowercase()
                    .contains(&showing.search.to_lowercase())
        })
        .map(|(name, key)| {
            let read_them_all = keys_of(reading, &name)
                .into_iter()
                .all(|other| readable(&reading.items[other]));
            (read_them_all, name, key)
        })
        .collect();
    rows.sort();
    rows.into_iter().map(|(_, _, key)| key).collect()
}
