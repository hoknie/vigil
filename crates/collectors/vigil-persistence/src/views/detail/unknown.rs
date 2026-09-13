use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::title;

pub(super) fn unknown(key: &str, _item: &Value, _reading: &Snapshot) -> Vec<Piece> {
    vec![
        title(key, false),
        Piece::Blank,
        Piece::text(
            "This console has no screen for this kind of object. Both halves ship in one \
             package, so this is a screen nobody has written yet, not an agent that ran \
             ahead. The key below is what a suppression matches.",
        ),
        Piece::Blank,
        Piece::field("object", key),
        Piece::Blank,
    ]
}
