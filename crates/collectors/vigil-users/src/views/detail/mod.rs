mod group;
mod key;
mod session;
mod session_source;
mod sudoer;
mod unknown;
mod user;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use crate::types::Kind;

pub(super) fn of(key: &str, item: &Value, reading: &Snapshot) -> Vec<Piece> {
    let mut said = match Kind::of(key) {
        Kind::Account => user::account(item, reading),
        Kind::Group => group::group(item, reading),
        Kind::Sudoer => sudoer::sudoer(item, reading),
        Kind::Key => key::key(item),
        Kind::Session => session::session(item),
        Kind::SessionSource => session_source::session_source(item),
        Kind::Unknown => unknown::unknown(key),
    };

    said.push(Piece::key(format!("user|{key}")));
    said
}

pub(super) fn title(name: &str, loud: bool) -> Piece {
    match loud {
        true => Piece::Warning(name.to_uppercase()),
        false => Piece::title("", name.to_uppercase()),
    }
}
