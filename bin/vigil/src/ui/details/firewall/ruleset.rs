use ratatui::text::{Line, Span};

use super::lines::{named, silencing};
use crate::ui::helpers::layout::{section, wrap};
use crate::ui::screens::firewall::Row;
use crate::ui::screens::firewall::fields;
use crate::ui::{Look, Report};

pub(super) fn ruleset(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled("NFTABLES", look.palette.accent()),
        Span::styled(
            format!(
                "  {}",
                fields::version(row.item).unwrap_or("version unknown")
            ),
            look.palette.heading(),
        ),
    ]));
    report.blank();

    for (name, value) in [
        ("tables", fields::tables(row.item).to_string()),
        ("chains", fields::all_chains(row.item).to_string()),
        ("on a hook", fields::base_chains(row.item).to_string()),
        ("on input", fields::hooked_on_input(row.item).to_string()),
        ("rules", fields::all_rules(row.item).to_string()),
        (
            "families",
            match fields::families(row.item).is_empty() {
                true => "none".to_string(),
                false => fields::families(row.item).join(", "),
            },
        ),
        ("object", row.key.clone()),
    ] {
        named(report, look, name, &value, width);
    }
    report.blank();

    report.push(section::rule(look, "WHAT THIS MEANS", width));
    let said = match (
        fields::legacy_backend(row.item),
        fields::hooked_on_input(row.item),
    ) {
        (true, _) => {
            "nftables lists no table here and the legacy iptables backend has tables \
                      registered. The rules are in the old backend, which this build reads as \
                      a marker and does not parse. What this host filters is not visible \
                      here, and that is not the same as a host that filters nothing."
        }
        (false, 0) => {
            "No chain is on the input hook. Nothing in this ruleset decides what \
                       happens to a packet arriving at this host, so every listening socket \
                       on the ports screen is reachable by anything that can route to it."
        }
        (false, _) => {
            "A chain on the input hook decides what happens to a packet no rule in \
                       it allowed. Its policy is on the row for that chain."
        }
    };
    for line in wrap::wrap(said, width.saturating_sub(5)) {
        report.push(Line::raw(format!("   {line}")));
    }
    report.blank();

    silencing(report, &row.key, look, width);
}
