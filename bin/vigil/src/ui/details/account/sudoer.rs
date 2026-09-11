use serde_json::Value;

use super::lines::{headline, named, say};
use crate::ui::helpers::layout::section;
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::{Kind, Row};
use crate::ui::{Look, Report, View};

pub(super) fn sudoer(report: &mut Report, row: &Row<'_>, view: &View, look: Look, width: usize) {
    let item = row.item;
    let who = accounts::text(item, "who").unwrap_or("?");
    headline(report, look, who, true);

    named(
        report,
        look,
        "grants",
        match item.get("all_commands").and_then(Value::as_bool) == Some(true) {
            true => "every command",
            false => "some commands",
        },
        width,
    );
    named(
        report,
        look,
        "password",
        match item.get("nopasswd").and_then(Value::as_bool) == Some(true) {
            true => "not required",
            false => "required",
        },
        width,
    );
    report.blank();

    report.push(section::rule(look, "WHERE IT IS WRITTEN", width));
    for rule in accounts::rules(item) {
        named(
            report,
            look,
            "file",
            accounts::text(rule, "source").unwrap_or("?"),
            width,
        );
        named(
            report,
            look,
            "rule",
            accounts::text(rule, "spec").unwrap_or("?"),
            width,
        );
        if rule.get("spec_redacted").and_then(Value::as_bool) == Some(true) {
            say(
                report,
                look.palette.quiet(),
                "Part of that rule was hidden on this host before it was written down.",
                width,
            );
        }
    }
    report.blank();

    report.push(section::rule(look, "WHO IT REACHES", width));
    let reached: Vec<&str> = accounts::objects(view, Kind::Account)
        .filter_map(|(_, account)| accounts::text(account, "name"))
        .filter(|name| {
            accounts::sudo_for(view, name)
                .iter()
                .any(|(_, grant)| accounts::text(grant, "who") == Some(who))
        })
        .collect();
    if reached.is_empty() {
        named(report, look, "accounts", "nobody in this reading", width);
    }
    for name in reached {
        named(report, look, "account", name, width);
    }
    report.blank();
}
