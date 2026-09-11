use serde_json::Value;

use super::lines::{headline, named, say};
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::Row;
use crate::ui::{Look, Report};

pub(super) fn session(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let item = row.item;
    headline(
        report,
        look,
        accounts::text(item, "user").unwrap_or("?"),
        item.get("remote").and_then(Value::as_bool) == Some(true),
    );

    named(
        report,
        look,
        "terminal",
        match accounts::text(item, "line") {
            Some(line) if !line.is_empty() => line,
            _ => "none: this session holds no terminal",
        },
        width,
    );
    named(
        report,
        look,
        "from",
        match accounts::text(item, "from") {
            Some(from) if !from.is_empty() => from,
            _ => "this host's console",
        },
        width,
    );
    named(report, look, "what", &accounts::what(item), width);
    if let Some(service) = accounts::text(item, "service").filter(|it| !it.is_empty()) {
        named(report, look, "opened by", service, width);
    }
    if let Some(id) = accounts::text(item, "session_id").filter(|it| !it.is_empty()) {
        named(report, look, "logind id", id, width);
    }
    named(report, look, "pid", &accounts::number(item, "pid"), width);
    named(report, look, "seen by", &accounts::seen_by(item), width);

    if !accounts::attended(item) {
        say(
            report,
            look.palette.quiet(),
            "Nobody is at a terminal in this session: it is either closing or one the host \
             opened for itself. It is listed because leaving it out would hide a way in.",
            width,
        );
    }
    report.blank();
}
