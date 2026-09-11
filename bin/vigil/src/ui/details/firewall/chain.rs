use ratatui::text::{Line, Span};

use super::lines::{named, silencing};
use crate::ui::helpers::layout::{section, wrap};
use crate::ui::screens::firewall::fields::{ACCEPT, DROP};
use crate::ui::screens::firewall::{Kind, Row};
use crate::ui::{Look, Report};

pub(super) fn chain(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let family = row.item["family"].as_str().unwrap_or("?");
    let table = row.item["table"].as_str().unwrap_or("?");
    let name = row.item["name"].as_str().unwrap_or("?");
    let policy = row.item["policy"].as_str().unwrap_or("?");

    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled("CHAIN", look.palette.accent()),
        Span::styled(
            format!("  {family} {table} · {name}"),
            look.palette.heading(),
        ),
    ]));
    report.blank();

    for (label, value) in [
        ("family", family.to_string()),
        ("table", table.to_string()),
        ("name", name.to_string()),
        ("type", row.item["type"].as_str().unwrap_or("?").to_string()),
        ("hook", row.item["hook"].as_str().unwrap_or("?").to_string()),
        (
            "priority",
            row.item["priority"]
                .as_i64()
                .map(|it| it.to_string())
                .unwrap_or_else(|| "—".to_string()),
        ),
        ("policy", policy.to_string()),
        (
            "rules",
            row.item["rules"]
                .as_u64()
                .map(|it| it.to_string())
                .unwrap_or_else(|| "—".to_string()),
        ),
        ("object", row.key.clone()),
    ] {
        named(report, look, label, &value, width);
    }
    report.blank();

    report.push(section::rule(look, "WHAT THE POLICY MEANS", width));
    let said = match policy {
        DROP => {
            "drop: a packet this chain saw and no rule in it allowed is thrown away. The \
                 chain is closed by default and the rules open it."
        }
        ACCEPT => {
            "accept: a packet this chain saw and no rule in it dropped goes through. The \
                   chain is open by default and the rules close it."
        }
        _ => {
            "This chain is not a base chain, or the reading carries no policy for it: a \
              regular chain is only reached by a jump from one that is."
        }
    };
    for line in wrap::wrap(said, width.saturating_sub(5)) {
        report.push(Line::styled(
            format!("   {line}"),
            match policy == ACCEPT && row.item["hook"].as_str() == Some("input") {
                true => look.palette.alarm(),
                false => look.palette.quiet(),
            },
        ));
    }
    report.blank();

    silencing(report, &row.key, look, width);
}

pub(super) fn backend(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let _ = Kind::Backend;
    let tables: Vec<&str> = row.item["tables"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect()
        })
        .unwrap_or_default();

    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled("BACKEND", look.palette.accent()),
        Span::styled("  legacy iptables", look.palette.heading()),
    ]));
    report.blank();

    named(
        report,
        look,
        "tables",
        &match tables.is_empty() {
            true => "none named".to_string(),
            false => tables.join(", "),
        },
        width,
    );
    named(
        report,
        look,
        "read from",
        "/proc/net/ip_tables_names",
        width,
    );
    named(report, look, "object", &row.key, width);
    report.blank();

    report.push(section::rule(look, "WHAT THIS MEANS", width));
    for line in wrap::wrap(
        "The nftables ruleset on this host is empty and the legacy iptables backend has \
         tables registered with the kernel. The rules are there and this build does not \
         parse them: it reads the marker and stops. The agent never calls a host in this \
         state a host without a firewall.",
        width.saturating_sub(5),
    ) {
        report.push(Line::styled(format!("   {line}"), look.palette.alarm()));
    }
    report.blank();

    silencing(report, &row.key, look, width);
}
