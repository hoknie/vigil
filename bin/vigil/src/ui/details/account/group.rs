use serde_json::Value;

use super::lines::{headline, named, say};
use crate::ui::helpers::layout::section;
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::{Kind, Row};
use crate::ui::{Look, Report, View};

pub(super) fn group(report: &mut Report, row: &Row<'_>, view: &View, look: Look, width: usize) {
    let item = row.item;
    let name = accounts::text(item, "name").unwrap_or("?");
    headline(report, look, name, accounts::privileged(item));

    named(report, look, "gid", &accounts::number(item, "gid"), width);
    report.blank();

    report.push(section::rule(look, "WHAT IT GRANTS", width));
    match accounts::text(item, "privilege") {
        Some(why) => say(report, look.palette.alarm(), why, width),
        None => say(
            report,
            look.palette.quiet(),
            "Nothing administrative in this reading.",
            width,
        ),
    }
    for (_, grant) in accounts::objects(view, Kind::Sudoer)
        .filter(|(_, grant)| accounts::text(grant, "who") == Some(&format!("%{name}")))
    {
        named(
            report,
            look,
            "sudo",
            match grant.get("all_commands").and_then(Value::as_bool) == Some(true) {
                true => "every command, to every member of this group",
                false => "some commands, to every member of this group",
            },
            width,
        );
    }
    report.blank();

    report.push(section::rule(look, "WHO IS IN IT", width));
    let members: Vec<&str> = accounts::members(item).collect();
    if members.is_empty() {
        named(report, look, "members", "none in this reading", width);
    }
    for member in members {
        named(report, look, "member", member, width);
    }
    report.blank();
}
