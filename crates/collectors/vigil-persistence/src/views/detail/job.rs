use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::{flag, text};
use super::title;

pub(super) fn job(_key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    let mut said = vec![
        title(
            text(item, "user").unwrap_or("?"),
            flag(item, "command_redacted"),
        ),
        Piece::Blank,
    ];

    said.push(Piece::field("from", text(item, "source").unwrap_or("?")));
    said.push(Piece::field("who", text(item, "user").unwrap_or("?")));
    said.push(Piece::field("when", text(item, "schedule").unwrap_or("?")));
    said.push(Piece::field(
        "command",
        text(item, "command").unwrap_or("?"),
    ));
    if flag(item, "command_redacted") {
        said.push(Piece::text(
            "Part of this command was hidden on this host before it was written down.",
        ));
    }
    said.push(Piece::Blank);
    said.push(Piece::text(
        "The whole command is part of the key below, on purpose: a line number moves when a \
         job is inserted above it, and a suppression nobody can read is a suppression nobody \
         writes.",
    ));
    said.push(Piece::Blank);

    said
}
