use super::lines::{headline, named, say};
use crate::ui::screens::programs::{Row, flag, number, strings, text};
use crate::ui::{Look, Report};

pub(super) fn running(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let path = text(row.item, "exe").unwrap_or("?");
    headline(report, look, path, flag(row.item, "exe_deleted"));

    named(report, look, "path", path, width);
    named(
        report,
        look,
        "file",
        match flag(row.item, "exe_deleted") {
            true => "unlinked from disk while the process is still running",
            false => "on disk",
        },
        width,
    );
    named(
        report,
        look,
        "user",
        text(row.item, "user").unwrap_or("not in /etc/passwd"),
        width,
    );
    if let Some(uid) = number(row.item, "uid") {
        named(report, look, "uid", &uid.to_string(), width);
    }

    let parents = strings(row.item, "parents");
    named(
        report,
        look,
        "started by",
        &match parents.is_empty() {
            true => "nothing this agent could see".to_string(),
            false => parents.join(", "),
        },
        width,
    );
    if flag(row.item, "parents_truncated") {
        say(
            report,
            look.palette.quiet(),
            "There are more parents than the agent records; the list above is the first of them.",
            width,
        );
    }

    named(report, look, "command", &command(row), width);
    if flag(row.item, "writable_path") {
        say(
            report,
            look.palette.alarm(),
            "This program is running from a path anybody on this host can write to.",
            width,
        );
    }
    report.blank();
}

fn command(row: &Row<'_>) -> String {
    if flag(row.item, "cmdline_redacted") {
        return "hidden on this host before it was written down".to_string();
    }
    if flag(row.item, "cmdline_varies") {
        return "varies between the processes running this program".to_string();
    }
    text(row.item, "cmdline")
        .unwrap_or("not recorded")
        .to_string()
}
