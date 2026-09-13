use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::{flag, number, strings, text};
use super::title;
use crate::types::Kind;

pub(super) fn file(key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    let path = text(item, "path").unwrap_or(key);
    let mut said = vec![title(path, !flag(item, "present")), Piece::Blank];

    said.push(Piece::field(
        "what it is",
        match Kind::of(key) {
            Kind::Preload => "every process on this host loads what it names",
            _ => text(item, "family").unwrap_or("a file other processes execute"),
        },
    ));
    said.push(Piece::field(
        "on disk",
        match (flag(item, "present"), flag(item, "readable")) {
            (false, _) => "no",
            (true, true) => "yes, and readable",
            (true, false) => "yes, and the agent was not allowed to read it",
        },
    ));
    if let Some(digest) = text(item, "sha256") {
        said.push(Piece::field("sha256", digest));
    }
    if let Some(size) = number(item, "size") {
        said.push(Piece::field("size", format!("{size} bytes")));
    }
    if let Some(mode) = text(item, "mode") {
        said.push(Piece::field("mode", mode));
    }
    if let (Some(uid), Some(gid)) = (number(item, "uid"), number(item, "gid")) {
        said.push(Piece::field("owner", format!("{uid}:{gid}")));
    }

    if Kind::of(key) == Kind::Preload {
        said.push(Piece::Blank);
        let entries = strings(item, "entries");
        match entries.is_empty() {
            true => said.push(Piece::text(
                "It names no library, so nothing is being forced into every process.",
            )),
            false => {
                for entry in entries {
                    said.push(Piece::field("loads", entry));
                }
            }
        }
    }
    said.push(Piece::Blank);

    said
}
