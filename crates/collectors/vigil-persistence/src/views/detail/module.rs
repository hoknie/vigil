use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::{number, strings, text};
use super::title;
use crate::types::Kind;

pub(super) fn module(key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    if Kind::of(key).mark() {
        return vec![
            title(key, true),
            Piece::Blank,
            Piece::warning(
                text(item, "reason")
                    .unwrap_or("/proc/modules could not be read: loaded modules are not watched"),
            ),
            Piece::Blank,
        ];
    }

    let name = text(item, "name").unwrap_or(key);
    let mut said = vec![title(name, false), Piece::Blank];

    if let Some(size) = number(item, "size") {
        said.push(Piece::field("size", format!("{size} bytes")));
    }
    said.push(Piece::field("state", text(item, "state").unwrap_or("?")));
    said.push(Piece::field(
        "used by",
        match strings(item, "dependencies").join(", ") {
            empty if empty.is_empty() => "nothing".to_string(),
            used_by => used_by,
        },
    ));
    said.push(Piece::Blank);

    said
}
