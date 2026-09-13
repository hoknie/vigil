use serde_json::Value;
use vigil_view::Piece;

use super::title;
use crate::views::facts::{attended, seen_by, what};
use crate::views::fields::{number, text};

pub(super) fn session(item: &Value) -> Vec<Piece> {
    let mut said = vec![
        title(
            text(item, "user").unwrap_or("?"),
            item.get("remote").and_then(Value::as_bool) == Some(true),
        ),
        Piece::Blank,
    ];

    said.push(Piece::field(
        "terminal",
        match text(item, "line") {
            Some(line) if !line.is_empty() => line,
            _ => "none: this session holds no terminal",
        },
    ));
    said.push(Piece::field(
        "from",
        match text(item, "from") {
            Some(from) if !from.is_empty() => from,
            _ => "this host's console",
        },
    ));
    said.push(Piece::field("what", what(item)));
    if let Some(service) = text(item, "service").filter(|it| !it.is_empty()) {
        said.push(Piece::field("opened by", service));
    }
    if let Some(id) = text(item, "session_id").filter(|it| !it.is_empty()) {
        said.push(Piece::field("logind id", id));
    }
    said.push(Piece::field("pid", number(item, "pid")));
    said.push(Piece::field("seen by", seen_by(item)));

    if !attended(item) {
        said.push(Piece::text(
            "Nobody is at a terminal in this session: it is either closing or one the host \
             opened for itself. It is listed because leaving it out would hide a way in.",
        ));
    }
    said.push(Piece::Blank);

    said
}
