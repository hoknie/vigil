use serde_json::Value;
use vigil_view::Piece;

use super::super::fields;

const LISTED: usize = 64;

pub(super) fn application(key: &str, item: &Value) -> Vec<Piece> {
    let enabled = fields::enabled(item);
    let flag = |field: &str| match item[field].as_bool() {
        Some(true) => "yes",
        _ => "no",
    };
    let mut said = vec![
        Piece::title(
            "APPLICATION FIREWALL",
            match enabled {
                true => "on",
                false => "off",
            },
        ),
        Piece::Blank,
    ];

    for (name, value) in [
        ("blocks all", flag("blocks_all").to_string()),
        ("stealth", flag("stealth").to_string()),
        ("signed apps", flag("allows_signed").to_string()),
        ("downloaded", flag("allows_downloaded_signed").to_string()),
        ("object", key.to_string()),
    ] {
        said.push(Piece::field(name, value));
    }
    said.push(Piece::Blank);

    let applications = fields::applications(item);
    said.push(Piece::heading("PROGRAMS IT DECIDES FOR"));
    for (path, allowed) in applications.iter().take(LISTED) {
        said.push(Piece::field(
            match allowed {
                true => "allowed",
                false => "blocked",
            },
            *path,
        ));
    }
    if applications.len() > LISTED {
        said.push(Piece::text(format!(
            "and {} more on the row itself",
            applications.len() - LISTED
        )));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(match enabled {
        false => Piece::warning(
            "The Application Firewall is off: any program on this Mac that listens accepts \
             connections from anything that can route to it, unless pf drops them first.",
        ),
        true => Piece::text(
            "The Application Firewall decides by program, not by port: a program listed as \
             allowed accepts incoming connections, a blocked one does not, and with block \
             all on only what macOS itself needs does.",
        ),
    });
    said.push(Piece::Blank);

    said
}
