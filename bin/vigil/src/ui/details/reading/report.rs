use ratatui::text::{Line, Span};
use serde_json::Value;

use super::subject::Subject;
use crate::ui::helpers::layout::{field, section, wrap};
use crate::ui::helpers::words::size;
use crate::ui::{Look, Report};

const NAME_WIDTH: usize = 22;

pub(super) fn report(subject: &Subject<'_>, look: Look, width: usize) -> Report {
    let mut report = Report::default();

    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(subject.kind.to_uppercase(), look.palette.accent()),
        Span::styled(format!("  {}", subject.named), look.palette.heading()),
    ]));
    report.blank();

    for (name, value) in values(subject.item) {
        match value {
            Said::One(said) => named(&mut report, look, &name, &said, width),
            Said::Many(all) => {
                named(
                    &mut report,
                    look,
                    &name,
                    &format!("{} in all", all.len()),
                    width,
                );
                for one in all {
                    for part in wrap::wrap(&one, width.saturating_sub(NAME_WIDTH + 8)) {
                        report.push(Line::raw(format!(
                            "   {}· {part}",
                            " ".repeat(NAME_WIDTH + 1)
                        )));
                    }
                }
            }
        }
    }
    named(&mut report, look, "object", &subject.key, width);
    report.blank();

    if !subject.means.is_empty() {
        report.push(section::rule(look, "WHAT THIS IS", width));
        for sentence in &subject.means {
            for line in wrap::wrap(sentence, width.saturating_sub(5)) {
                report.push(Line::raw(format!("   {line}")));
            }
        }
        report.blank();
    }

    report
}

enum Said {
    One(String),
    Many(Vec<String>),
}

fn values(item: &Value) -> Vec<(String, Said)> {
    let Some(fields) = item.as_object() else {
        return vec![("value".to_string(), Said::One(plain(item)))];
    };

    fields
        .iter()
        .map(|(name, value)| {
            (
                name.replace('_', " "),
                match value.as_array() {
                    Some(all) => Said::Many(all.iter().map(plain).collect()),
                    None => Said::One(with_a_size(name, value)),
                },
            )
        })
        .collect()
}

fn with_a_size(name: &str, value: &Value) -> String {
    match (name.ends_with("_bytes"), value.as_u64()) {
        (true, Some(bytes)) => format!("{} ({bytes} bytes)", size::bytes(bytes)),
        _ => plain(value),
    }
}

fn plain(value: &Value) -> String {
    match value {
        Value::Null => "not read".to_string(),
        Value::String(said) => said.clone(),
        Value::Bool(yes) => match yes {
            true => "yes".to_string(),
            false => "no".to_string(),
        },
        other => other.to_string(),
    }
}

fn named(report: &mut Report, look: Look, name: &str, value: &str, width: usize) {
    for line in field::lines(look, name, value, NAME_WIDTH, width) {
        report.push(line);
    }
}
