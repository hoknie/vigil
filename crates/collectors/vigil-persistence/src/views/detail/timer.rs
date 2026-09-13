use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::super::fields::text;
use super::super::timers::schedules;
use super::title;

pub(super) fn timer(key: &str, item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    let name = text(item, "name").unwrap_or(key);
    let mut said = vec![title(name, false), Piece::Blank];

    said.push(Piece::field("path", text(item, "path").unwrap_or("?")));
    said.push(Piece::field(
        "description",
        text(item, "description").unwrap_or("none in the file"),
    ));
    for line in when(item) {
        said.push(Piece::field("when", line));
    }
    said.push(Piece::field(
        "activates",
        text(item, "activates").unwrap_or("?"),
    ));
    said.push(Piece::Blank);

    said
}

fn when(item: &Value) -> Vec<String> {
    let calendar = schedules(item);
    if !calendar.is_empty() {
        return calendar.into_iter().map(str::to_string).collect();
    }
    match text(item, "on_boot") {
        Some(after) => vec![format!("{after} after boot or after its own last run")],
        None => vec!["not stated in the file".to_string()],
    }
}
