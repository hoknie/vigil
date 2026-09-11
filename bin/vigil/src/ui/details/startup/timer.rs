use super::lines::{headline, named};
use crate::ui::screens::programs::text;
use crate::ui::screens::startup::{Row, schedules};
use crate::ui::{Look, Report};

pub(super) fn timer(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let name = text(row.item, "name").unwrap_or(&row.key);
    headline(report, look, name, false);

    named(
        report,
        look,
        "path",
        text(row.item, "path").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "description",
        text(row.item, "description").unwrap_or("none in the file"),
        width,
    );
    for line in when(row) {
        named(report, look, "when", &line, width);
    }
    named(
        report,
        look,
        "activates",
        text(row.item, "activates").unwrap_or("?"),
        width,
    );
    report.blank();
}

fn when(row: &Row<'_>) -> Vec<String> {
    let calendar = schedules(row);
    if !calendar.is_empty() {
        return calendar.into_iter().map(str::to_string).collect();
    }
    match text(row.item, "on_boot") {
        Some(after) => vec![format!("{after} after boot or after its own last run")],
        None => vec!["not stated in the file".to_string()],
    }
}
