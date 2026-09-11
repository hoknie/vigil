use super::lines::{headline, named, say};
use crate::ui::screens::programs::{number, strings, text};
use crate::ui::screens::startup::Row;
use crate::ui::{Look, Report};

pub(super) fn module(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    if row.mark() {
        headline(report, look, &row.key, true);
        say(
            report,
            look.palette.alarm(),
            text(row.item, "reason")
                .unwrap_or("/proc/modules could not be read: loaded modules are not watched"),
            width,
        );
        report.blank();
        return;
    }

    let name = text(row.item, "name").unwrap_or(&row.key);
    headline(report, look, name, false);

    if let Some(size) = number(row.item, "size") {
        named(report, look, "size", &format!("{size} bytes"), width);
    }
    named(
        report,
        look,
        "state",
        text(row.item, "state").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "used by",
        &match strings(row.item, "dependencies").join(", ") {
            empty if empty.is_empty() => "nothing".to_string(),
            used_by => used_by,
        },
        width,
    );
    report.blank();
}
