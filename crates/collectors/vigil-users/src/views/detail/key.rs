use serde_json::Value;
use vigil_view::Piece;

use super::title;
use crate::views::facts::readable;
use crate::views::fields::text;

pub(super) fn key(item: &Value) -> Vec<Piece> {
    let user = text(item, "user").unwrap_or("?");
    let mut said = vec![title(user, true), Piece::Blank];

    if !readable(item) {
        said.push(Piece::warning(format!(
            "{} was not readable. That is a refusal, not an account with no keys: there may \
             be keys here that were never read.",
            text(item, "source").unwrap_or("This file")
        )));
        said.push(Piece::Blank);
        return said;
    }

    said.push(Piece::field(
        "algorithm",
        text(item, "algorithm").unwrap_or("?"),
    ));
    said.push(Piece::field(
        "comment",
        text(item, "comment").unwrap_or("(none)"),
    ));
    said.push(Piece::field(
        "restricted",
        text(item, "options").unwrap_or("no command, source or expiry restriction on this key"),
    ));
    said.push(Piece::field("file", text(item, "source").unwrap_or("?")));
    said.push(Piece::Blank);

    said.push(Piece::heading("FINGERPRINT"));
    said.push(Piece::field(
        "",
        text(item, "fingerprint").unwrap_or("no fingerprint"),
    ));
    said.push(Piece::text(
        "Compare it with `ssh-keygen -lf` on the key you believe should be there.",
    ));
    said.push(Piece::Blank);

    said
}
