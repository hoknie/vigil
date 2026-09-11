use ratatui::text::{Line, Span};

use crate::ui::helpers::finding::suppression;
use crate::ui::helpers::layout::field;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::layout::wrap;
use crate::ui::types::focus::anchor;
use crate::ui::{Look, Report};

pub(super) fn silencing(report: &mut Report, key: &str, look: Look, width: usize) {
    report.push(section::rule(look, "SUPPRESS", width));
    let finding_key = format!("{}|{key}", anchor::ACCOUNTS);
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

pub(super) fn headline(report: &mut Report, look: Look, name: &str, loud: bool) {
    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(
            name.to_uppercase(),
            match loud {
                true => look.palette.alarm(),
                false => look.palette.heading(),
            },
        ),
    ]));
    report.blank();
}

pub(super) fn named(report: &mut Report, look: Look, name: &str, value: &str, width: usize) {
    for line in field::lines(look, name, value, 10, width) {
        report.push(line);
    }
}

pub(super) fn say(report: &mut Report, style: ratatui::style::Style, sentence: &str, width: usize) {
    for line in wrap::wrap(sentence, width.saturating_sub(5)) {
        report.push(Line::styled(format!("   {line}"), style));
    }
}
