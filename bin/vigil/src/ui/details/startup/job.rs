use super::lines::{headline, named, say};
use crate::ui::screens::programs::{flag, text};
use crate::ui::screens::startup::Row;
use crate::ui::{Look, Report};

pub(super) fn job(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    headline(
        report,
        look,
        text(row.item, "user").unwrap_or("?"),
        flag(row.item, "command_redacted"),
    );

    named(
        report,
        look,
        "from",
        text(row.item, "source").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "who",
        text(row.item, "user").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "when",
        text(row.item, "schedule").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "command",
        text(row.item, "command").unwrap_or("?"),
        width,
    );
    if flag(row.item, "command_redacted") {
        say(
            report,
            look.palette.quiet(),
            "Part of this command was hidden on this host before it was written down.",
            width,
        );
    }
    report.blank();
    say(
        report,
        look.palette.quiet(),
        "The whole command is part of the key below, on purpose: a line number moves when a job \
         is inserted above it, and a suppression nobody can read is a suppression nobody writes.",
        width,
    );
    report.blank();
}
