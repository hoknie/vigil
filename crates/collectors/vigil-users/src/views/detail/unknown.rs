use vigil_view::Piece;

use super::title;

pub(super) fn unknown(key: &str) -> Vec<Piece> {
    vec![
        title(key, false),
        Piece::Blank,
        Piece::text(
            "This console has no screen for this kind of object. Both halves ship in one \
             package, so this is a screen nobody has written yet, not an agent that ran \
             ahead. The key above is what a suppression matches.",
        ),
        Piece::Blank,
    ]
}
