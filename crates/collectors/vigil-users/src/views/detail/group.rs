use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::title;
use crate::types::Kind;
use crate::views::facts::privileged;
use crate::views::fields::{members, number, objects, text};

pub(super) fn group(item: &Value, reading: &Snapshot) -> Vec<Piece> {
    let name = text(item, "name").unwrap_or("?");
    let mut said = vec![title(name, privileged(item)), Piece::Blank];

    said.push(Piece::field("gid", number(item, "gid")));
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT IT GRANTS"));
    match text(item, "privilege") {
        Some(why) => said.push(Piece::warning(why)),
        None => said.push(Piece::text("Nothing administrative in this reading.")),
    }
    for (_, grant) in objects(reading, Kind::Sudoer)
        .filter(|(_, grant)| text(grant, "who") == Some(&format!("%{name}")))
    {
        said.push(Piece::field(
            "sudo",
            match grant.get("all_commands").and_then(Value::as_bool) == Some(true) {
                true => "every command, to every member of this group",
                false => "some commands, to every member of this group",
            },
        ));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHO IS IN IT"));
    let members: Vec<&str> = members(item).collect();
    if members.is_empty() {
        said.push(Piece::field("members", "none in this reading"));
    }
    for member in members {
        said.push(Piece::field("member", member));
    }
    said.push(Piece::Blank);

    said
}
