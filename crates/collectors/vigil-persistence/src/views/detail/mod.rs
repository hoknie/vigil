mod file;
mod job;
mod launchd;
mod module;
mod timer;
mod unit;
mod unknown;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use crate::types::Kind;

pub(super) fn of(key: &str, item: &Value, reading: &Snapshot) -> Vec<Piece> {
    let mut said = match Kind::of(key) {
        Kind::Launchd => launchd::job(key, item, reading),
        Kind::Unit => unit::unit(key, item, reading),
        Kind::Timer => timer::timer(key, item, reading),
        Kind::Cron => job::job(key, item, reading),
        Kind::Module | Kind::ModulesUnreadable => module::module(key, item, reading),
        Kind::Script | Kind::Preload => file::file(key, item, reading),
        Kind::Unknown => unknown::unknown(key, item, reading),
    };

    said.push(Piece::key(format!("persistence|{key}")));
    said
}

pub(super) fn title(name: &str, loud: bool) -> Piece {
    match loud {
        true => Piece::Warning(name.to_uppercase()),
        false => Piece::title("", name.to_uppercase()),
    }
}
