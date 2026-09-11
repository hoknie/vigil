use serde_json::Value;

use super::fields::{members, objects, text};
use super::kind::Kind;
use crate::ui::View;

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

pub fn groups_of<'a>(view: &'a View, name: &str) -> Vec<(&'a str, &'a Value)> {
    objects(view, Kind::Group)
        .filter(|(_, group)| members(group).any(|member| member == name))
        .collect()
}

pub fn sudo_for<'a>(view: &'a View, name: &str) -> Vec<(&'a str, &'a Value)> {
    let groups: Vec<&str> = groups_of(view, name)
        .into_iter()
        .filter_map(|(_, group)| text(group, "name"))
        .collect();

    objects(view, Kind::Sudoer)
        .filter(|(_, grant)| match text(grant, "who") {
            Some(who) => match who.strip_prefix('%') {
                Some(group) => groups.contains(&group),
                None => who == name,
            },
            None => false,
        })
        .collect()
}

pub fn route_to_root(view: &View, account: &Value) -> String {
    let name = text(account, "name").unwrap_or_default();

    let mut routes: Vec<String> = Vec::new();
    if account.get("uid").and_then(Value::as_u64) == Some(0) {
        routes.push("uid 0".to_string());
    }
    for (_, group) in groups_of(view, name) {
        if privileged(group)
            && let Some(group) = text(group, "name")
        {
            routes.push(group.to_string());
        }
    }
    let grants = sudo_for(view, name);
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
