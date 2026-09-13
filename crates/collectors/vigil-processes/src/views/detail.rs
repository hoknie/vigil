use serde_json::Value;
use vigil_view::Piece;

use super::fields::{flag, number, strings, text};

pub(super) fn running(item: &Value) -> Vec<Piece> {
    let mut said: Vec<Piece> = Vec::new();
    let path = text(item, "exe").unwrap_or("?");
    said.push(title(path, flag(item, "exe_deleted")));

    said.push(Piece::field("path", path));
    said.push(Piece::field(
        "file",
        match flag(item, "exe_deleted") {
            true => "unlinked from disk while the process is still running",
            false => "on disk",
        },
    ));
    said.push(Piece::field(
        "user",
        text(item, "user").unwrap_or("not in /etc/passwd"),
    ));
    if let Some(uid) = number(item, "uid") {
        said.push(Piece::field("uid", uid.to_string()));
    }

    let parents = strings(item, "parents");
    said.push(Piece::field(
        "started by",
        match parents.is_empty() {
            true => "nothing this agent could see".to_string(),
            false => parents.join(", "),
        },
    ));
    if flag(item, "parents_truncated") {
        said.push(Piece::text(
            "There are more parents than the agent records; the list above is the first of them.",
        ));
    }

    said.push(Piece::field("command", command(item)));
    if flag(item, "writable_path") {
        said.push(Piece::warning(
            "This program is running from a path anybody on this host can write to.",
        ));
    }
    said.push(Piece::Blank);
    said
}

fn command(item: &Value) -> String {
    if flag(item, "cmdline_redacted") {
        return "hidden on this host before it was written down".to_string();
    }
    if flag(item, "cmdline_varies") {
        return "varies between the processes running this program".to_string();
    }
    text(item, "cmdline").unwrap_or("not recorded").to_string()
}

fn title(name: &str, loud: bool) -> Piece {
    match loud {
        true => Piece::Warning(name.to_uppercase()),
        false => Piece::title("", name.to_uppercase()),
    }
}
