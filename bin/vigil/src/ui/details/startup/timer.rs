use super::lines::{headline, named};
use crate::ui::screens::programs::{flag, text};
use crate::ui::screens::startup::Row;
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
    named(
        report,
        look,
        "when",
        match text(row.item, "on_calendar") {
            Some(calendar) => calendar,
            None => match flag(row.item, "on_boot") {
                true => "on boot",
                false => "not stated in the file",
            },
        },
        width,
    );
    named(
        report,
        look,
        "activates",
        text(row.item, "activates").unwrap_or("?"),
        width,
    );
    report.blank();
}
