use serde_json::Value;
use vigil_model::Snapshot;

use super::fields::{members, objects, text, under};
use crate::parsers::PRIVILEGED_GROUPS;
use crate::types::Kind;

pub fn could_log_in(account: &Value) -> bool {
    account.get("uid").and_then(Value::as_u64) == Some(0)
        || account.get("interactive").and_then(Value::as_bool) == Some(true)
        || account
            .get("password_permits_login")
            .and_then(Value::as_bool)
            == Some(true)
        || account.get("shadow_readable").and_then(Value::as_bool) != Some(true)
}

pub fn password(account: &Value) -> String {
    if account.get("shadow_readable").and_then(Value::as_bool) != Some(true) {
        return "unknown".to_string();
    }
    text(account, "password").unwrap_or("unknown").to_string()
}

pub fn privileged(group: &Value) -> bool {
    group.get("privileged").and_then(Value::as_bool) == Some(true)
}

pub fn readable(key: &Value) -> bool {
    key.get("readable").and_then(Value::as_bool) == Some(true)
}

pub(super) fn remote(session: &Value) -> bool {
    session.get("remote").and_then(Value::as_bool) == Some(true)
}

pub fn attended(session: &Value) -> bool {
    session.get("attended").and_then(Value::as_bool) != Some(false)
}

pub fn answers(source: &Value) -> bool {
    source.get("answers").and_then(Value::as_bool) == Some(true)
}

pub fn what(session: &Value) -> String {
    let said: Vec<&str> = ["class", "state", "type"]
        .iter()
        .filter_map(|field| text(session, field))
        .filter(|value| !value.is_empty() && *value != "unspecified")
        .collect();

    match said.is_empty() {
        true => "a login this host recorded and said no more about".to_string(),
        false => said.join(" · "),
    }
}

pub fn seen_by(session: &Value) -> String {
    let sources: Vec<&str> = session
        .get("seen_by")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .collect();

    match sources.is_empty() {
        true => "?".to_string(),
        false => sources.join(", "),
    }
}

fn listed_groups(account: &Value) -> Option<&Vec<Value>> {
    account.get("groups").and_then(Value::as_array)
}

pub fn groups_for<'a>(reading: &'a Snapshot, account: &Value) -> Vec<(&'a str, &'a Value)> {
    let name = text(account, "name").unwrap_or_default();
    let Some(listed) = listed_groups(account) else {
        return objects(reading, Kind::Group)
            .filter(|(_, group)| members(group).any(|member| member == name))
            .collect();
    };

    let mut found: Vec<(&str, &Value)> = listed
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|group| {
            reading
                .items
                .get_key_value(&format!("{}{group}", Kind::Group.prefix()))
        })
        .map(|(key, group)| (key.as_str(), group))
        .collect();
    found.sort_unstable_by(|left, right| left.0.cmp(right.0));
    found.dedup_by(|left, right| left.0 == right.0);
    found
}

pub fn group_named<'a>(reading: &'a Snapshot, group: &str) -> Option<&'a Value> {
    reading
        .items
        .get(&format!("{}{group}", Kind::Group.prefix()))
}

fn in_group(reading: &Snapshot, group: &str, name: &str) -> bool {
    group_named(reading, group).is_some_and(|found| members(found).any(|member| member == name))
}

pub fn keys_of<'a>(
    reading: &'a Snapshot,
    user: &str,
) -> impl Iterator<Item = (&'a str, &'a Value)> {
    let user = user.to_string();
    under(reading, format!("{}{user}|", Kind::Key.prefix()))
        .filter(move |(_, key)| text(key, "user") == Some(user.as_str()))
}

pub fn sessions_of<'a>(reading: &'a Snapshot, user: &str) -> Vec<(&'a str, &'a Value)> {
    under(reading, format!("{}{user}|", Kind::Session.prefix()))
        .filter(|(_, session)| text(session, "user") == Some(user))
        .collect()
}

pub fn grant_to_group<'a>(reading: &'a Snapshot, group: &str) -> Option<&'a Value> {
    let who = format!("%{group}");
    reading
        .items
        .get(&format!("{}{who}", Kind::Sudoer.prefix()))
        .filter(|grant| text(grant, "who") == Some(who.as_str()))
}

pub fn reached_by<'a>(reading: &'a Snapshot, who: &str) -> Vec<&'a str> {
    let mut named: Vec<&str> = match who.strip_prefix('%') {
        Some(group) => group_named(reading, group)
            .map(|found| members(found).collect())
            .unwrap_or_default(),
        None => vec![who],
    };
    named.sort_unstable();
    named.dedup();

    named
        .into_iter()
        .filter_map(|name| {
            reading
                .items
                .get(&format!("{}{name}", Kind::Account.prefix()))
                .and_then(|account| text(account, "name"))
                .filter(|found| *found == name)
        })
        .collect()
}

fn grants_to_groups_of<'a>(reading: &'a Snapshot, account: &Value) -> Vec<(&'a str, &'a Value)> {
    let prefix = Kind::Sudoer.prefix();
    let name = text(account, "name").unwrap_or_default();

    match listed_groups(account) {
        Some(_) => groups_for(reading, account)
            .into_iter()
            .filter_map(|(key, _)| key.strip_prefix(Kind::Group.prefix()))
            .filter_map(|group| {
                let who = format!("%{group}");
                reading
                    .items
                    .get_key_value(&format!("{prefix}{who}"))
                    .filter(|(_, grant)| text(grant, "who") == Some(who.as_str()))
            })
            .map(|(key, grant)| (key.as_str(), grant))
            .collect(),
        None => under(reading, format!("{prefix}%"))
            .filter(|(_, grant)| {
                text(grant, "who")
                    .and_then(|who| who.strip_prefix('%'))
                    .is_some_and(|group| in_group(reading, group, name))
            })
            .collect(),
    }
}

pub fn sudo_for<'a>(reading: &'a Snapshot, account: &Value) -> Vec<(&'a str, &'a Value)> {
    let prefix = Kind::Sudoer.prefix();
    let name = text(account, "name").unwrap_or_default();

    let mut found = grants_to_groups_of(reading, account);
    if let Some((key, grant)) = reading.items.get_key_value(&format!("{prefix}{name}"))
        && text(grant, "who") == Some(name)
    {
        found.push((key.as_str(), grant));
    }
    found.sort_by(|left, right| left.0.cmp(right.0));
    found
}

fn privileged_groups_held<'a>(reading: &'a Snapshot, account: &Value) -> Vec<&'a str> {
    let name = text(account, "name").unwrap_or_default();
    let mut held: Vec<&str> = match listed_groups(account) {
        Some(_) => groups_for(reading, account)
            .into_iter()
            .filter(|(_, group)| privileged(group))
            .map(|(key, group)| {
                text(group, "name")
                    .unwrap_or_else(|| key.strip_prefix(Kind::Group.prefix()).unwrap_or(key))
            })
            .collect(),
        None => PRIVILEGED_GROUPS
            .iter()
            .map(|(group, _)| *group)
            .filter(|group| {
                group_named(reading, group).is_some_and(|found| {
                    privileged(found) && members(found).any(|member| member == name)
                })
            })
            .collect(),
    };
    held.sort_unstable();
    held
}

pub fn route_to_root(reading: &Snapshot, account: &Value) -> String {
    let mut routes: Vec<String> = Vec::new();
    if account.get("uid").and_then(Value::as_u64) == Some(0) {
        routes.push("uid 0".to_string());
    }
    routes.extend(
        privileged_groups_held(reading, account)
            .into_iter()
            .map(str::to_string),
    );
    let grants = sudo_for(reading, account);
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
