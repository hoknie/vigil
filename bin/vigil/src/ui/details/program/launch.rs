use super::lines::{headline, named, say};
use crate::ui::screens::programs::{Row, flag, number, text};
use crate::ui::{Look, Report};

pub(super) fn launch(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let path = text(row.item, "exe").unwrap_or("?");
    headline(report, look, path, flag(row.item, "writable_path"));

    named(report, look, "path", path, width);
    named(
        report,
        look,
        "who",
        text(row.item, "user").unwrap_or("no name for that login id"),
        width,
    );
    if let Some(auid) = number(row.item, "auid") {
        named(report, look, "login id", &auid.to_string(), width);
    }
    if let Some(id) = number(row.item, "audit_id") {
        named(report, look, "audit id", &id.to_string(), width);
    }
    named(
        report,
        look,
        "first seen",
        text(row.item, "first_seen").unwrap_or("?"),
        width,
    );
    named(report, look, "arguments", &arguments(row), width);

    named(
        report,
        look,
        "the file",
        match flag(row.item, "exe_present") {
            true => "was on disk when this record was read",
            false => "was not on disk when this record was read",
        },
        width,
    );
    say(
        report,
        look.palette.quiet(),
        "That answers what was true at the moment the record was read, not what is true now: \
         a row here is frozen when it first appears and is never taken off the list.",
        width,
    );
    if flag(row.item, "writable_path") {
        say(
            report,
            look.palette.alarm(),
            "It was run from a path anybody on this host can write to.",
            width,
        );
    }
    report.blank();
}

fn arguments(row: &Row<'_>) -> String {
    if flag(row.item, "arguments_redacted") {
        return "hidden on this host before they were written down".to_string();
    }
    text(row.item, "arguments")
        .unwrap_or("not recorded: the agent is not told to keep launch arguments")
        .to_string()
}
