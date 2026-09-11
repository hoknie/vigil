use serde_json::Value;

use super::lines::{headline, named, say};
use crate::ui::helpers::layout::section;
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::{Kind, Row};
use crate::ui::{Look, Report, View};

pub(super) fn account(report: &mut Report, row: &Row<'_>, view: &View, look: Look, width: usize) {
    let item = row.item;
    let name = accounts::text(item, "name").unwrap_or("?");
    headline(report, look, name, accounts::could_log_in(item));

    named(report, look, "uid", &accounts::number(item, "uid"), width);
    named(report, look, "gid", &accounts::number(item, "gid"), width);
    named(
        report,
        look,
        "shell",
        accounts::text(item, "shell").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "home",
        accounts::text(item, "home").unwrap_or("?"),
        width,
    );
    report.blank();

    report.push(section::rule(look, "PASSWORD", width));
    if item.get("shadow_readable").and_then(Value::as_bool) != Some(true) {
        say(
            report,
            look.palette.alarm(),
            "unknown: this account's line in /etc/shadow was not readable. That is not the \
             same as an account with no password.",
            width,
        );
    } else {
        named(report, look, "state", &accounts::password(item), width);
        if let Some(day) = item.get("password_last_change_day").and_then(Value::as_u64) {
            named(report, look, "changed", &format!("day {day}"), width);
        }
        named(
            report,
            look,
            "lets in",
            match item.get("password_permits_login").and_then(Value::as_bool) {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            },
            width,
        );
    }
    report.blank();

    report.push(section::rule(look, "WHAT IT CAN REACH", width));
    let routes = accounts::route_to_root(view, item);
    match routes.is_empty() {
        true => say(
            report,
            look.palette.quiet(),
            "Nothing administrative reaches this account: no uid 0, no privileged group, no \
             sudo grant.",
            width,
        ),
        false => say(
            report,
            look.palette.alarm(),
            &format!("root by {routes}"),
            width,
        ),
    }

    let groups = accounts::groups_of(view, name);
    if groups.is_empty() {
        named(report, look, "groups", "none in this reading", width);
    }
    for (key, group) in &groups {
        let label = accounts::text(group, "name").unwrap_or(key);
        match accounts::text(group, "privilege") {
            Some(why) => {
                named(report, look, "group", &format!("{label}: {why}"), width);
            }
            None => named(report, look, "group", label, width),
        }
    }

    let grants = accounts::sudo_for(view, name);
    if grants.is_empty() {
        named(report, look, "sudo", "nothing grants it sudo", width);
    }
    for (_, grant) in &grants {
        let who = accounts::text(grant, "who").unwrap_or("?");
        let through = match who.starts_with('%') {
            true => format!("{who} (through the group)"),
            false => who.to_string(),
        };
        named(report, look, "sudo", &through, width);
        for rule in accounts::rules(grant) {
            named(
                report,
                look,
                "",
                &format!(
                    "{}: {}",
                    accounts::text(rule, "source").unwrap_or("?"),
                    accounts::text(rule, "spec").unwrap_or("?")
                ),
                width,
            );
            if rule.get("spec_redacted").and_then(Value::as_bool) == Some(true) {
                say(
                    report,
                    look.palette.quiet(),
                    "Part of that rule was hidden on this host before it was written down.",
                    width,
                );
            }
        }
    }
    report.blank();

    report.push(section::rule(
        look,
        "KEYS THAT LOG IN WITHOUT A PASSWORD",
        width,
    ));
    let keys: Vec<(&str, &Value)> = accounts::objects(view, Kind::Key)
        .filter(|(_, key)| accounts::text(key, "user") == Some(name))
        .collect();
    if keys.is_empty() {
        named(report, look, "keys", "none in this reading", width);
    }
    for (_, item) in &keys {
        match accounts::readable(item) {
            false => say(
                report,
                look.palette.alarm(),
                &format!(
                    "{} was not readable. This account may have keys that were never read.",
                    accounts::text(item, "source").unwrap_or("Its authorized_keys")
                ),
                width,
            ),
            true => {
                named(
                    report,
                    look,
                    "key",
                    &format!(
                        "{} {}",
                        accounts::text(item, "algorithm").unwrap_or("?"),
                        accounts::text(item, "comment").unwrap_or("(no comment)")
                    ),
                    width,
                );
                named(
                    report,
                    look,
                    "",
                    accounts::text(item, "fingerprint").unwrap_or("no fingerprint"),
                    width,
                );
                named(
                    report,
                    look,
                    "",
                    match accounts::text(item, "options") {
                        Some(options) => options,
                        None => "no command, source or expiry restriction",
                    },
                    width,
                );
            }
        }
    }
    report.blank();

    report.push(section::rule(look, "LOGGED IN NOW", width));
    let sessions: Vec<(&str, &Value)> = accounts::objects(view, Kind::Session)
        .filter(|(_, session)| accounts::text(session, "user") == Some(name))
        .collect();
    match (sessions.is_empty(), view.collector_note("users")) {
        (true, Some(_)) => say(
            report,
            look.palette.quiet(),
            "No login recorded for this account, and this reading is incomplete. The tally \
             under the list says what the collector was refused.",
            width,
        ),
        (true, None) => named(report, look, "sessions", "none", width),
        (false, _) => {
            for (_, session) in &sessions {
                named(
                    report,
                    look,
                    "session",
                    &format!(
                        "{} from {} (pid {}, seen by {})",
                        match accounts::text(session, "line") {
                            Some(line) if !line.is_empty() => line,
                            _ => "no terminal",
                        },
                        match accounts::text(session, "from") {
                            Some(from) if !from.is_empty() => from,
                            _ => "this host's console",
                        },
                        accounts::number(session, "pid"),
                        accounts::seen_by(session)
                    ),
                    width,
                );
            }
        }
    }
    report.blank();
}
