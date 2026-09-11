use super::lines::{headline, named, say};
use crate::ui::screens::programs::{flag, strings, text};
use crate::ui::screens::startup::Row;
use crate::ui::{Look, Report};

const PULLED_IN_BY: &[(&str, &str)] = &[
    ("wanted_by", "WantedBy"),
    ("required_by", "RequiredBy"),
    ("part_of", "PartOf"),
];

const PULLS_IN: &[(&str, &str)] = &[("wants", "Wants"), ("requires", "Requires")];

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

    for line in pulled(row) {
        named(report, look, &line.0, &line.1, width);
    }

    for command in strings(row.item, "commands") {
        named(report, look, "runs", command, width);
    }
    if flag(row.item, "commands_redacted") {
        say(
            report,
            look.palette.quiet(),
            "Part of what this unit runs was hidden on this host before it was written down.",
            width,
        );
    }
    report.blank();
}

fn pulled(row: &Row<'_>) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    for (field, setting) in PULLED_IN_BY {
        for name in strings(row.item, field) {
            lines.push(("pulled by".to_string(), format!("{name} · {setting}")));
        }
    }
    if lines.is_empty() {
        lines.push((
            "pulled by".to_string(),
            "nothing in these unit files: it is a root of the tree".to_string(),
        ));
    }
    for (field, setting) in PULLS_IN {
        for name in strings(row.item, field) {
            lines.push(("pulls in".to_string(), format!("{name} · {setting}")));
        }
    }
    lines
}
