use super::lines::{headline, named, say};
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::Row;
use crate::ui::{Look, Report};

pub(super) fn session_source(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let item = row.item;
    headline(
        report,
        look,
        accounts::text(item, "source").unwrap_or("?"),
        !accounts::answers(item),
    );

    named(
        report,
        look,
        "read from",
        accounts::text(item, "path").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "on host",
        match accounts::flag(item, "present") {
            true => "yes",
            false => "no",
        },
        width,
    );
    named(
        report,
        look,
        "was read",
        match accounts::flag(item, "read") {
            true => "yes",
            false => "no",
        },
        width,
    );
    named(
        report,
        look,
        "sessions",
        &accounts::number(item, "sessions"),
        width,
    );

    if let Some(reason) = accounts::text(item, "reason") {
        say(report, look.palette.quiet(), reason, width);
    }
    report.blank();
}
