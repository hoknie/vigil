use super::lines::{headline, named, say};
use crate::ui::screens::programs::{flag, number, strings, text};
use crate::ui::screens::startup::{Kind, Row};
use crate::ui::{Look, Report};

pub(super) fn file(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let path = text(row.item, "path").unwrap_or(&row.key);
    headline(report, look, path, !flag(row.item, "present"));

    named(
        report,
        look,
        "what it is",
        match row.kind {
            Kind::Preload => "every process on this host loads what it names",
            _ => text(row.item, "family").unwrap_or("a file other processes execute"),
        },
        width,
    );
    named(
        report,
        look,
        "on disk",
        match (flag(row.item, "present"), flag(row.item, "readable")) {
            (false, _) => "no",
            (true, true) => "yes, and readable",
            (true, false) => "yes, and the agent was not allowed to read it",
        },
        width,
    );
    if let Some(digest) = text(row.item, "sha256") {
        named(report, look, "sha256", digest, width);
    }
    if let Some(size) = number(row.item, "size") {
        named(report, look, "size", &format!("{size} bytes"), width);
    }
    if let Some(mode) = text(row.item, "mode") {
        named(report, look, "mode", mode, width);
    }
    if let (Some(uid), Some(gid)) = (number(row.item, "uid"), number(row.item, "gid")) {
        named(report, look, "owner", &format!("{uid}:{gid}"), width);
    }

    if row.kind == Kind::Preload {
        report.blank();
        let entries = strings(row.item, "entries");
        match entries.is_empty() {
            true => say(
                report,
                look.palette.quiet(),
                "It names no library, so nothing is being forced into every process.",
                width,
            ),
            false => {
                for entry in entries {
                    named(report, look, "loads", entry, width);
                }
            }
        }
    }
    report.blank();
}
