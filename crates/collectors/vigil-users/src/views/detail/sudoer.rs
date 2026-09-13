use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::facts::sudo_for;
use super::super::fields::{objects, rules, text};
use crate::types::Kind;

use super::title;

pub(super) fn sudoer(item: &Value, reading: &Snapshot) -> Vec<Piece> {
    let mut said: Vec<Piece> = Vec::new();
    let who = text(item, "who").unwrap_or("?");
    said.push(title(who, true));

    said.push(Piece::field(
        "grants",
        match item.get("all_commands").and_then(Value::as_bool) == Some(true) {
            true => "every command",
            false => "some commands",
        },
    ));
    said.push(Piece::field(
        "password",
        match item.get("nopasswd").and_then(Value::as_bool) == Some(true) {
            true => "not required",
            false => "required",
        },
    ));
    said.push(Piece::Blank);

    said.push(Piece::heading("WHERE IT IS WRITTEN"));
    for rule in rules(item) {
        said.push(Piece::field("file", text(rule, "source").unwrap_or("?")));
        said.push(Piece::field("rule", text(rule, "spec").unwrap_or("?")));
        if rule.get("spec_redacted").and_then(Value::as_bool) == Some(true) {
            said.push(Piece::text(
                "Part of that rule was hidden on this host before it was written down.",
            ));
        }
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHO IT REACHES"));
    let reached: Vec<&str> = objects(reading, Kind::Account)
        .filter_map(|(_, account)| text(account, "name"))
        .filter(|name| {
            sudo_for(reading, name)
                .iter()
                .any(|(_, grant)| text(grant, "who") == Some(who))
        })
        .collect();
    if reached.is_empty() {
        said.push(Piece::field("accounts", "nobody in this reading"));
    }
    for name in reached {
        said.push(Piece::field("account", name));
    }
    said.push(Piece::Blank);
    said
}
