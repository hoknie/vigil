use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::title;
use crate::views::facts::{
    could_log_in, groups_for, keys_of, password, readable, route_to_root, seen_by, sessions_of,
    sudo_for,
};
use crate::views::fields::{number, rules, text};

pub(super) fn account(item: &Value, reading: &Snapshot) -> Vec<Piece> {
    let name = text(item, "name").unwrap_or("?");
    let mut said = vec![title(name, could_log_in(item)), Piece::Blank];

    said.push(Piece::field("uid", number(item, "uid")));
    said.push(Piece::field("gid", number(item, "gid")));
    said.push(Piece::field("shell", text(item, "shell").unwrap_or("?")));
    said.push(Piece::field("home", text(item, "home").unwrap_or("?")));
    said.push(Piece::Blank);

    said.push(Piece::heading("PASSWORD"));
    if item.get("shadow_readable").and_then(Value::as_bool) != Some(true) {
        said.push(Piece::warning(
            "unknown: this account's line in /etc/shadow was not readable. That is not the \
             same as an account with no password.",
        ));
    } else {
        said.push(Piece::field("state", password(item)));
        if let Some(day) = item.get("password_last_change_day").and_then(Value::as_u64) {
            said.push(Piece::field("changed", format!("day {day}")));
        }
        said.push(Piece::field(
            "lets in",
            match item.get("password_permits_login").and_then(Value::as_bool) {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            },
        ));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT IT CAN REACH"));
    let routes = route_to_root(reading, item);
    match routes.is_empty() {
        true => said.push(Piece::text(
            "Nothing administrative reaches this account: no uid 0, no privileged group, no \
             sudo grant.",
        )),
        false => said.push(Piece::warning(format!("root by {routes}"))),
    }

    let groups = groups_for(reading, item);
    if groups.is_empty() {
        said.push(Piece::field("groups", "none in this reading"));
    }
    for (key, group) in &groups {
        let label = text(group, "name").unwrap_or(key);
        said.push(Piece::field(
            "group",
            match text(group, "privilege") {
                Some(why) => format!("{label}: {why}"),
                None => label.to_string(),
            },
        ));
    }

    let grants = sudo_for(reading, item);
    if grants.is_empty() {
        said.push(Piece::field("sudo", "nothing grants it sudo"));
    }
    for (_, grant) in &grants {
        let who = text(grant, "who").unwrap_or("?");
        said.push(Piece::field(
            "sudo",
            match who.starts_with('%') {
                true => format!("{who} (through the group)"),
                false => who.to_string(),
            },
        ));
        for rule in rules(grant) {
            said.push(Piece::field(
                "",
                format!(
                    "{}: {}",
                    text(rule, "source").unwrap_or("?"),
                    text(rule, "spec").unwrap_or("?")
                ),
            ));
            if rule.get("spec_redacted").and_then(Value::as_bool) == Some(true) {
                said.push(Piece::text(
                    "Part of that rule was hidden on this host before it was written down.",
                ));
            }
        }
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("KEYS THAT LOG IN WITHOUT A PASSWORD"));
    let keys: Vec<(&str, &Value)> = keys_of(reading, name).collect();
    if keys.is_empty() {
        said.push(Piece::field("keys", "none in this reading"));
    }
    for (_, item) in &keys {
        match readable(item) {
            false => said.push(Piece::warning(format!(
                "{} was not readable. This account may have keys that were never read.",
                text(item, "source").unwrap_or("Its authorized_keys")
            ))),
            true => {
                said.push(Piece::field(
                    "key",
                    format!(
                        "{} {}",
                        text(item, "algorithm").unwrap_or("?"),
                        text(item, "comment").unwrap_or("(no comment)")
                    ),
                ));
                said.push(Piece::field(
                    "",
                    text(item, "fingerprint").unwrap_or("no fingerprint"),
                ));
                said.push(Piece::field(
                    "",
                    text(item, "options").unwrap_or("no command, source or expiry restriction"),
                ));
            }
        }
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("LOGGED IN NOW"));
    let sessions = sessions_of(reading, name);
    match sessions.is_empty() {
        true => said.push(Piece::field("sessions", "none")),
        false => {
            for (_, session) in &sessions {
                said.push(Piece::field(
                    "session",
                    format!(
                        "{} from {} (pid {}, seen by {})",
                        match text(session, "line") {
                            Some(line) if !line.is_empty() => line,
                            _ => "no terminal",
                        },
                        match text(session, "from") {
                            Some(from) if !from.is_empty() => from,
                            _ => "this host's console",
                        },
                        number(session, "pid"),
                        seen_by(session)
                    ),
                ));
            }
        }
    }
    said.push(Piece::Blank);

    said
}
