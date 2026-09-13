use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::{flag, strings, text};
use super::title;

const PULLED_IN_BY: &[(&str, &str)] = &[
    ("wanted_by", "WantedBy"),
    ("required_by", "RequiredBy"),
    ("part_of", "PartOf"),
];

const PULLS_IN: &[(&str, &str)] = &[("wants", "Wants"), ("requires", "Requires")];

pub(super) fn unit(key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    let mut said: Vec<Piece> = Vec::new();
    let name = text(item, "name").unwrap_or(key);
    said.push(title(name, !flag(item, "readable")));

    said.push(Piece::field("path", text(item, "path").unwrap_or("?")));
    said.push(Piece::field(
        "description",
        text(item, "description").unwrap_or("none in the file"),
    ));
    said.push(Piece::field(
        "runs as",
        text(item, "run_as").unwrap_or("root"),
    ));
    said.push(Piece::field(
        "readable",
        match flag(item, "readable") {
            true => "yes",
            false => "no: the commands below are what could be read, which may be none",
        },
    ));

    for line in pulled(item) {
        said.push(Piece::field(&line.0, &line.1));
    }

    for command in strings(item, "commands") {
        said.push(Piece::field("runs", command));
    }
    if flag(item, "commands_redacted") {
        said.push(Piece::text(
            "Part of what this unit runs was hidden on this host before it was written down.",
        ));
    }
    said.push(Piece::Blank);
    said
}

fn pulled(item: &Value) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    for (field, setting) in PULLED_IN_BY {
        for name in strings(item, field) {
            lines.push(("pulled by".to_string(), format!("{name} · {setting}")));
        }
    }
    if lines.is_empty() {
        lines.push((
            "pulled by".to_string(),
            "nothing in these unit files: it is a root of the tree".to_string(),
        ));
    }
    for (field, setting) in PULLS_IN {
        for name in strings(item, field) {
            lines.push(("pulls in".to_string(), format!("{name} · {setting}")));
        }
    }
    lines
}
