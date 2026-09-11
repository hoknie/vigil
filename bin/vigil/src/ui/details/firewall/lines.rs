use ratatui::text::Line;

use crate::ui::helpers::finding::suppression;
use crate::ui::helpers::layout::{field, section};
use crate::ui::types::focus::anchor;
use crate::ui::{Look, Report};

const NAME_WIDTH: usize = 10;

const SHORT: &str = "fw-";

pub(super) fn named(report: &mut Report, look: Look, name: &str, value: &str, width: usize) {
    for line in field::lines(look, name, value, NAME_WIDTH, width) {
        report.push(line);
    }
}

pub(super) fn silencing(report: &mut Report, key: &str, look: Look, width: usize) {
    report.push(section::rule(look, "SUPPRESS", width));
    let finding_key = format!(
        "{}|{}",
        anchor::FIREWALL,
        key.strip_prefix(SHORT).unwrap_or(key)
    );
    for line in suppression::entry(&finding_key, None) {
        report.push(Line::raw(format!("   {line}")));
    }
    if finding_key.chars().count() + 21 > width {
        report.push(Line::styled(
            "   widen the terminal before copying the key above".to_string(),
            look.palette.alarm(),
        ));
    }
}
