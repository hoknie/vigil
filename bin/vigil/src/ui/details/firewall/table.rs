use ratatui::text::{Line, Span};

use super::lines::{named, silencing};
use crate::ui::helpers::layout::{section, wrap};
use crate::ui::screens::firewall::Row;
use crate::ui::{Look, Report};

pub(super) fn table(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let family = row.item["family"].as_str().unwrap_or("?");
    let name = row.item["name"].as_str().unwrap_or("?");

    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled("TABLE", look.palette.accent()),
        Span::styled(format!("  {family} {name}"), look.palette.heading()),
    ]));
    report.blank();

    for (label, value) in [
        ("family", family.to_string()),
        ("name", name.to_string()),
        ("chains", number(row, "chains")),
        ("rules", number(row, "rules")),
        ("object", row.key.clone()),
    ] {
        named(report, look, label, &value, width);
    }
    report.blank();

    report.push(section::rule(look, "WHAT THIS MEANS", width));
    for line in wrap::wrap(
        "A table holds chains, and the chains hold the rules. The number here is every rule \
         in the table, counted; the rules themselves are not read, because fail2ban and \
         docker rewrite theirs continually and a row per rule would be a finding on every \
         one of those.",
        width.saturating_sub(5),
    ) {
        report.push(Line::raw(format!("   {line}")));
    }
    report.blank();

    silencing(report, &row.key, look, width);
}

fn number(row: &Row<'_>, field: &str) -> String {
    row.item[field]
        .as_u64()
        .map(|count| count.to_string())
        .unwrap_or_else(|| "—".to_string())
}
