use serde_json::Value;
use vigil_view::Piece;

use super::fields::{flag, number, text};

pub(super) fn launch(item: &Value) -> Vec<Piece> {
    let mut said: Vec<Piece> = Vec::new();
    let path = text(item, "exe").unwrap_or("?");
    said.push(title(path, flag(item, "writable_path")));

    said.push(Piece::field("path", path));
    said.push(Piece::field(
        "who",
        text(item, "user").unwrap_or("no name for that login id"),
    ));
    if let Some(auid) = number(item, "auid") {
        said.push(Piece::field("login id", auid.to_string()));
    }
    if let Some(id) = number(item, "audit_id") {
        said.push(Piece::field("audit id", id.to_string()));
    }
    said.push(Piece::field(
        "first seen",
        text(item, "first_seen").unwrap_or("?"),
    ));
    said.push(Piece::field("arguments", arguments(item)));

    said.push(Piece::field(
        "the file",
        match flag(item, "exe_present") {
            true => "was on disk when this record was read",
            false => "was not on disk when this record was read",
        },
    ));
    said.push(Piece::text(
        "That answers what was true at the moment the record was read, not what is true now: \
         a row here is frozen when it first appears and is never taken off the list.",
    ));
    if flag(item, "writable_path") {
        said.push(Piece::warning(
            "It was run from a path anybody on this host can write to.",
        ));
    }
    said.push(Piece::Blank);
    said
}

fn arguments(item: &Value) -> String {
    if flag(item, "arguments_redacted") {
        return "hidden on this host before they were written down".to_string();
    }
    text(item, "arguments")
        .unwrap_or("not recorded: the agent is not told to keep launch arguments")
        .to_string()
}

fn title(name: &str, loud: bool) -> Piece {
    match loud {
        true => Piece::Warning(name.to_uppercase()),
        false => Piece::title("", name.to_uppercase()),
    }
}
