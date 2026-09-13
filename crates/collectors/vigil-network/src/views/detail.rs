use serde_json::Value;
use vigil_view::{Piece, basename};

use super::fields::{endpoint, protocol, user};
use crate::types::SocketView;

pub(super) fn socket(key: &str, item: &Value) -> Vec<Piece> {
    let view = SocketView::new(item);
    let mut pieces = vec![
        Piece::title(protocol(item, key).to_uppercase(), endpoint(item, key)),
        Piece::Blank,
    ];

    for (name, value) in [
        ("protocol", protocol(item, key).to_string()),
        ("address", text(item, "address")),
        ("port", number(item, "port")),
        ("path", text(item, "path")),
        ("user", user(item)),
        ("uid", number(item, "uid")),
        ("object", key.to_string()),
    ] {
        if value.is_empty() {
            continue;
        }
        pieces.push(Piece::field(name, value));
    }
    pieces.push(Piece::Blank);
    pieces.push(Piece::heading("WHAT IS HOLDING IT"));

    match view.owner_resolved() {
        false => {
            pieces.push(Piece::warning(
                "Not resolved. The socket is there and the process behind it was out of \
                 reach. That is not the same as nothing holding it. The summary screen says \
                 what the collector was refused.",
            ));
            pieces.push(Piece::Blank);
        }
        true => {
            pieces.push(Piece::field(
                "program",
                view.executable().unwrap_or("unknown"),
            ));
            if view.executable_deleted() {
                pieces.push(Piece::warning(
                    "That file has been unlinked while the process is still running.",
                ));
            }
            if let Some(line) = view.command_line() {
                pieces.push(Piece::field("command", line));
            }
            if view.command_line_redacted() {
                pieces.push(Piece::text(
                    "Part of this command line was hidden on this host before it was written \
                     down.",
                ));
            }
            pieces.push(Piece::Blank);
        }
    }

    pieces.push(Piece::key(format!("port.listen|{key}")));
    pieces
}

pub(super) fn program(path: &str, sockets: usize) -> Vec<Piece> {
    vec![
        Piece::title("", basename(path)),
        Piece::Blank,
        Piece::field("program", path),
        Piece::field("sockets", sockets.to_string()),
        Piece::Blank,
        Piece::text(
            "The sockets are listed under this heading. Press Esc for the list, then move to \
             one of them for its detail.",
        ),
    ]
}

pub(super) fn unresolved(sockets: usize) -> Vec<Piece> {
    vec![
        Piece::Warning("OWNER NOT RESOLVED".to_string()),
        Piece::Blank,
        Piece::field("sockets", sockets.to_string()),
        Piece::Blank,
        Piece::warning(
            "These are not a program called \"unknown\", and not sockets with no owner. \
             Resolving one means reading /proc/<pid>/fd for every process on the host; that \
             was refused. The summary screen says which privilege is missing.",
        ),
    ]
}

fn text(item: &Value, name: &str) -> String {
    item.get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn number(item: &Value, name: &str) -> String {
    item.get(name)
        .and_then(Value::as_u64)
        .map(|number| number.to_string())
        .unwrap_or_default()
}
