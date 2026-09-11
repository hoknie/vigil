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
use crate::ui::{Look, Notice, Report};
pub fn render(finding: Option<&Finding>, look: Look, top: usize, area: Rect, buffer: &mut Buffer) {
    let Some(finding) = finding else {
        Notice::plain("No finding is selected.")
            .saying("Move to a finding and press Enter. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(finding, look, look.text_width(area.width));
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
        Some(finding) => report(finding, look, width).len(),
        None => 0,
    }
}

fn report(finding: &Finding, look: Look, width: usize) -> Report {
    let mut report = Report::default();

    headline(&mut report, finding, look, width);
    changes(&mut report, finding, look, width);
    evidence(&mut report, finding, look, width);
    redacted(&mut report, finding, look, width);
    silencing(&mut report, finding, look, width);

    report
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
    if finding.finding_key.chars().count() + 21 > width {
        report.push(Line::styled(
            "   widen the terminal before copying the key above".to_string(),
            look.palette.alarm(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::{Evidence, Severity};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(finding: Option<&Finding>) -> String {
        drawn_at(finding, 80)
    }

    fn drawn_at(finding: Option<&Finding>, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 60));
        render(finding, fixture::look(), 0, buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn it_shows_what_the_object_was_and_what_it_became() {
        let mut finding = fixture::finding("The owner of :443 changed", Severity::High);
        finding.before = Some(json!({"process": {"exe": "/usr/sbin/nginx"}, "user": "root"}));
        finding.after = Some(json!({"process": {"exe": "/tmp/.x/nc"}, "user": "www-data"}));

        let page = drawn(Some(&finding));

        assert!(page.contains("- process.exe"), "{page}");
        assert!(page.contains("/usr/sbin/nginx"), "{page}");
        assert!(page.contains("+ process.exe"), "{page}");
        assert!(page.contains("/tmp/.x/nc"), "{page}");
    }

    #[test]
    fn what_was_hidden_before_writing_is_named_rather_than_shown_as_empty() {
        let mut finding = fixture::finding("A new listening port", Severity::Critical);
        finding.after = Some(json!({"process": {"cmdline": "mysql -p<hidden>"}}));
        finding.redacted = vec!["/after/process/cmdline".into()];

        let page = drawn(Some(&finding));

        assert!(page.contains("HIDDEN"), "{page}");
        assert!(page.contains("/after/process/cmdline"), "{page}");
    }

    #[test]
    fn evidence_is_on_the_screen_because_it_is_what_makes_a_finding_investigable() {
        let mut finding = fixture::finding("A new listening port", Severity::Critical);
        finding.evidence = vec![Evidence {
            kind: "path".into(),
            value: "/tmp/.x/nc".into(),
        }];

        assert!(drawn(Some(&finding)).contains("/tmp/.x/nc"));
    }

    #[test]
    fn it_spells_the_configuration_entry_that_would_silence_this_finding() {
        let mut finding = fixture::finding("A group gained a member", Severity::High);
        finding.finding_key = "user|group|docker".into();

        let page = drawn(Some(&finding));

        assert!(page.contains("suppressions:"), "{page}");
        assert!(
            page.contains("finding_key: \"user|group|docker\""),
            "{page}"
        );
        assert!(page.contains("reason:"), "{page}");
    }

    #[test]
    fn a_long_object_key_is_shown_whole_rather_than_cut_where_nobody_can_see_the_cut() {
        let mut finding = fixture::finding("A key was added", Severity::High);
        finding.finding_key = format!("user|sshkey|deploy|SHA256:{}", "a".repeat(60));

        let page = drawn(Some(&finding));
        let unbroken: String = page.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            unbroken.contains(&finding.finding_key),
            "the key is not all there: {page}"
        );
        for line in page.lines() {
            assert!(line.chars().count() <= 80, "{line}");
        }
    }

    #[test]
    fn a_long_title_is_wrapped_because_the_title_is_the_point_of_the_screen() {
        let mut finding = fixture::finding(
            "A new SSH key may log in as intruder: ssh-ed25519 \
             SHA256:YpTjnHs8O4g1NgWt/Kcm+JLcyjBkWlA25hV/c7zRKRU person@laptop",
            Severity::High,
        );
        finding.finding_key = "user|sshkey|intruder".into();

        let page = drawn(Some(&finding));

        assert!(
            page.contains("person@laptop"),
            "the end of it survived: {page}"
        );
        for line in page.lines() {
            assert!(line.chars().count() <= 80, "{line}");
        }
    }

    #[test]
    fn with_nothing_selected_it_says_how_to_select_something() {
        assert!(drawn(None).contains("press Enter"));
    }

    #[test]
    fn the_page_is_as_long_as_it_is_so_the_scroll_position_can_be_kept_inside_it() {
        let finding = fixture::finding("A new listening port", Severity::Critical);

        assert!(height(Some(&finding), fixture::look(), 80) > 10);
        assert_eq!(height(None, fixture::look(), 80), 0);
    }

    #[test]
    fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
        let mut finding = fixture::finding("A new listening port", Severity::Critical);
        finding.after = Some(json!({"process": {"cmdline": "a".repeat(300)}}));

        for width in [80u16, 120, 200] {
            for line in drawn_at(Some(&finding), width).lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}
