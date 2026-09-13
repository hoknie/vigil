use serde_json::Value;
use vigil_view::Piece;

use super::title;
use crate::views::facts::answers;
use crate::views::fields::{flag, number, text};

pub(super) fn session_source(item: &Value) -> Vec<Piece> {
    let mut said = vec![
        title(text(item, "source").unwrap_or("?"), !answers(item)),
        Piece::Blank,
    ];

    said.push(Piece::field("read from", text(item, "path").unwrap_or("?")));
    said.push(Piece::field(
        "on host",
        match flag(item, "present") {
            true => "yes",
            false => "no",
        },
    ));
    said.push(Piece::field(
        "was read",
        match flag(item, "read") {
            true => "yes",
            false => "no",
        },
    ));
    said.push(Piece::field("sessions", number(item, "sessions")));

    if let Some(reason) = text(item, "reason") {
        said.push(Piece::text(reason));
    }
    said.push(Piece::Blank);

    said
}
