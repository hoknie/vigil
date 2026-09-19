use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::{flag, strings, text};
use super::super::lists::launchd::whose;
use super::title;

pub(super) fn job(key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    let inserted = strings(item, "inserted_libraries");
    let loud = !flag(item, "readable") || flag(item, "writable_path") || !inserted.is_empty();
    let mut said = vec![title(text(item, "name").unwrap_or(key), loud), Piece::Blank];

    said.push(Piece::field("path", text(item, "path").unwrap_or("?")));
    said.push(Piece::field(
        "kind",
        match text(item, "domain") {
            Some("agent") => "agent: started in the session of a person who logs in",
            _ => "daemon: started by launchd for the whole Mac",
        },
    ));
    said.push(Piece::field("whose", whose(item)));
    said.push(Piece::field("runs as", text(item, "run_as").unwrap_or("?")));
    said.push(Piece::field("when", text(item, "schedule").unwrap_or("—")));
    said.push(Piece::field(
        "program",
        text(item, "program").unwrap_or("none named in the file"),
    ));
    for command in strings(item, "commands") {
        said.push(Piece::field("runs", command));
    }
    for library in &inserted {
        said.push(Piece::field("loads first", *library));
    }
    if flag(item, "disabled") {
        said.push(Piece::field(
            "disabled",
            "the file says Disabled; launchctl enable overrides it",
        ));
    }
    if let Some(refusal) = text(item, "refusal") {
        said.push(Piece::field("unread", refusal));
    }
    if flag(item, "commands_redacted") {
        said.push(Piece::text(
            "Part of what this job runs was hidden on this Mac before it was written down.",
        ));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(match (flag(item, "writable_path"), inserted.is_empty()) {
        (true, _) => Piece::warning(
            "The program this job starts lives where an account other than the one it runs as \
             can write, so whoever writes there decides what runs.",
        ),
        (false, false) => Piece::warning(
            "DYLD_INSERT_LIBRARIES loads the libraries above into the program before its own \
             code runs; it is how code is slipped into a program that is itself genuine.",
        ),
        (false, true) => Piece::text(
            "launchd starts this program by itself, as written in this file; the file's \
             place decides for whom, and its keys decide when.",
        ),
    });
    said.push(Piece::Blank);
    said
}
