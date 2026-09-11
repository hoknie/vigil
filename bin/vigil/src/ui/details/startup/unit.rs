use super::lines::{headline, named, say};
use crate::ui::screens::programs::{flag, strings, text};
use crate::ui::screens::startup::Row;
use crate::ui::{Look, Report};

pub(super) fn unit(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let name = text(row.item, "name").unwrap_or(&row.key);
    headline(report, look, name, !flag(row.item, "readable"));

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
        "runs as",
        text(row.item, "run_as").unwrap_or("root"),
        width,
    );
    named(
        report,
        look,
        "readable",
        match flag(row.item, "readable") {
            true => "yes",
            false => "no: the commands below are what could be read, which may be none",
        },
        width,
    );

    match flag(row.item, "commands_redacted") {
        true => say(
            report,
            look.palette.quiet(),
            "Part of what this unit runs was hidden on this host before it was written down.",
            width,
        ),
        false => {
            for command in strings(row.item, "commands") {
                named(report, look, "runs", command, width);
            }
        }
    }
    report.blank();
}
