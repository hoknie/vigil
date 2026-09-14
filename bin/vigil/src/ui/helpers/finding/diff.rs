use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};
use vigil_model::Finding;

use crate::ui::helpers::finding::change_lines;
use crate::ui::helpers::finding::suppression;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::field;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::layout::wrap;
use crate::ui::types::content::change;
use crate::ui::{Deed, Look, Notice, Report};
pub fn render(
    finding: Option<&Finding>,
    look: Look,
    top: usize,
    picked: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    let Some(finding) = finding else {
        Notice::plain("No finding is selected.")
            .saying("Move to a finding and press Enter. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(finding, look, look.text_width(area.width), picked);
    let page = area.height as usize;
    let top = top.min(report.len().saturating_sub(page));

    Paragraph::new(
        report
            .lines()
            .iter()
            .skip(top)
            .take(page)
            .cloned()
            .collect::<Vec<Line>>(),
    )
    .render(
        Rect {
            width: area.width - gutter,
            ..area
        },
        buffer,
    );

    if gutter > 0 && report.len() > page {
        let mut bar = ScrollbarState::new(report.len() - page).position(top);
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(look.palette.border())
                .begin_symbol(None)
                .end_symbol(None),
            area,
            buffer,
            &mut bar,
        );
    }
}
pub fn height(finding: Option<&Finding>, look: Look, width: usize) -> usize {
    match finding {
        Some(finding) => report(finding, look, width, 0).len(),
        None => 0,
    }
}

fn report(finding: &Finding, look: Look, width: usize, picked: usize) -> Report {
    let mut report = Report::default();

    headline(&mut report, finding, look, width);
    changes(&mut report, finding, look, width);
    evidence(&mut report, finding, look, width);
    redacted(&mut report, finding, look, width);
    actions(&mut report, look, width, picked);
    silencing(&mut report, finding, look, width);

    report
}

fn actions(report: &mut Report, look: Look, width: usize, picked: usize) {
    if !look.interactive() || Deed::ALL.is_empty() {
        return;
    }
    report.push(section::rule(look, "ACTIONS", width));
    for deed in Deed::ALL {
        report.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(deed.key().to_string(), look.palette.accent()),
            Span::raw(format!(
                "   {}",
                match picked {
                    0 => deed.alone().to_string(),
                    count => format!("{}, all {count} of them", deed.named()),
                }
            )),
        ]));
    }
    report.blank();
}

fn headline(report: &mut Report, finding: &Finding, look: Look, width: usize) {
    let level = finding.severity.as_str().to_uppercase();
    let indent = level.chars().count() + 5;
    for (index, part) in wrap::wrap(&finding.title, width.saturating_sub(indent))
        .into_iter()
        .enumerate()
    {
        report.push(match index {
            0 => Line::from(vec![
                Span::raw("   "),
                Span::styled(level.clone(), look.palette.severity(&finding.severity)),
                Span::styled(format!("  {part}"), look.palette.heading()),
            ]),
            _ => Line::styled(
                format!("{}{part}", " ".repeat(indent)),
                look.palette.heading(),
            ),
        });
    }
    report.push(Line::raw(""));

    for (name, value) in [
        ("kind", finding.kind.as_str().to_string()),
        ("object", finding.finding_key.clone()),
        (
            "seen",
            format!("{} ({} time(s))", finding.observed_at, finding.occurrences),
        ),
        ("first", finding.first_seen_at.clone()),
        (
            "state",
            match finding.state {
                vigil_model::State::Open => "open".to_string(),
                vigil_model::State::Resolved => "resolved".to_string(),
            },
        ),
        ("rule", finding.rule.clone().unwrap_or_default()),
        ("event", finding.event_id.clone()),
    ] {
        if value.is_empty() {
            continue;
        }
        for line in field::lines(look, name, &value, 8, width) {
            report.push(line);
        }
    }
    report.blank();
}

fn changes(report: &mut Report, finding: &Finding, look: Look, width: usize) {
    report.push(section::rule(look, "CHANGED", width));

    let changes = change_lines::lines(finding.before.as_ref(), finding.after.as_ref());
    if changes.is_empty() {
        report.push(Line::raw(
            "   nothing recorded on either side of this finding",
        ));
        report.blank();
        return;
    }
    let value_width = width.saturating_sub(31).max(12);
    for line in &changes {
        let mark = line.mark.symbol();
        for (index, part) in wrap::wrap(&line.value, value_width).into_iter().enumerate() {
            let text = match index {
                0 => format!("   {mark} {} {part}", column::fit(&line.path, 24)),
                _ => format!("     {} {part}", column::fit("", 24)),
            };
            report.push(match line.mark {
                change::Mark::Same => Line::styled(text, look.palette.quiet()),
                _ => Line::raw(text),
            });
        }
    }
    report.blank();
}

fn evidence(report: &mut Report, finding: &Finding, look: Look, width: usize) {
    if finding.evidence.is_empty() {
        return;
    }

    report.push(section::rule(look, "EVIDENCE", width));
    for evidence in &finding.evidence {
        for line in field::lines(look, &evidence.kind, &evidence.value, 10, width) {
            report.push(line);
        }
    }
    report.blank();
}

fn redacted(report: &mut Report, finding: &Finding, look: Look, width: usize) {
    if finding.redacted.is_empty() {
        return;
    }
    report.push(section::rule(
        look,
        "HIDDEN ON THIS HOST BEFORE ANYTHING WAS WRITTEN DOWN",
        width,
    ));
    for path in &finding.redacted {
        report.push(Line::raw(format!("   {path}")));
    }
    report.blank();
}
fn silencing(report: &mut Report, finding: &Finding, look: Look, width: usize) {
    report.push(section::rule(look, "SUPPRESS", width));
    for line in suppression::snippet(finding) {
        report.push(Line::raw(format!("   {line}")));
    }
    if suppression::widest(&finding.finding_key, Some(finding.kind.as_str())) + 3 > width {
        report.push(Line::styled(
            "   widen the terminal before copying the key above".to_string(),
            look.palette.alarm(),
        ));
    }
}
