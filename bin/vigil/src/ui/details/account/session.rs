use serde_json::Value;

use super::lines::{headline, named};
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
        accounts::text(item, "line").unwrap_or("?"),
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
    named(report, look, "pid", &accounts::number(item, "pid"), width);
    report.blank();
}
