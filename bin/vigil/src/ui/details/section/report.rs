use ratatui::text::{Line, Span};

use crate::ui::helpers::layout::{field, section, wrap};
use crate::ui::screens::home::Row;
use crate::ui::{Look, Report};

const NAME_WIDTH: usize = 9;

const ABOUT: &str = "ABOUT";

const WARNING: &str = "WARNING";

pub fn report(row: &Row, look: Look, width: usize) -> Report {
    let mut report = Report::default();

    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(row.name.to_uppercase(), look.palette.accent()),
        Span::styled(format!("  {}", row.holds), look.palette.heading()),
    ]));
    report.blank();

    for (name, value) in [
        ("state", row.standing.state.clone()),
        (
            "collector",
            match row.collector.is_empty() {
                true => "none: this section is made of what the others read".to_string(),
                false => row.collector.clone(),
            },
        ),
        (
            "objects",
            match row.standing.objects {
                Some(count) => count.to_string(),
                None => "—".to_string(),
            },
        ),
        (
            "read",
            row.standing.read.clone().unwrap_or_else(|| "—".to_string()),
        ),
        (
            "opens",
            match row.number {
                Some(number) => number.to_string(),
                None => "nothing: this console has no section for it".to_string(),
            },
        ),
    ] {
        for line in field::lines(look, name, &value, NAME_WIDTH, width) {
            report.push(line);
        }
    }
    let Some(note) = &row.standing.note else {
        return report;
    };

    report.blank();
    report.push(section::rule(
        look,
        match row.standing.unwell {
            true => WARNING,
            false => ABOUT,
        },
        width,
    ));
    for line in wrap::wrap(note, width.saturating_sub(5)) {
        report.push(Line::styled(
            format!("   {line}"),
            match row.standing.unwell {
                true => look.palette.alarm(),
                false => look.palette.quiet(),
            },
        ));
    }

    report
}
