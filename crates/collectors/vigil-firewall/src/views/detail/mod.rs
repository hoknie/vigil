mod chain;
mod interface;
mod rule;
mod ruleset;
mod table;
mod zone;

pub(in crate::views) use zone::zone;

use serde_json::Value;
use vigil_view::Piece;

use crate::types::Kind;

const SHORT: &str = "fw-";

pub(super) fn of(key: &str, item: &Value) -> Vec<Piece> {
    let mut said = match Kind::of(key) {
        Some(Kind::Ruleset) => ruleset::ruleset(key, item),
        Some(Kind::Table) => table::table(key, item),
        Some(Kind::Chain) => {
            let mut said = chain::chain(key, item);
            said.extend(rule::rules(item));
            said
        }
        Some(Kind::Backend) => chain::backend(key, item),
        Some(Kind::Interface) => interface::interface(key, item),
        None => Vec::new(),
    };

    said.push(Piece::key(format!(
        "firewall|{}",
        key.strip_prefix(SHORT).unwrap_or(key)
    )));
    said
}
