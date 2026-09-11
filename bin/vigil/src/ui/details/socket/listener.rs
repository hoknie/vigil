use ratatui::text::{Line, Span};
use serde_json::Value;

use super::lines::{named, silencing};
use crate::ui::helpers::layout::section;
use crate::ui::helpers::layout::wrap;
use crate::ui::screens::ports;
use crate::ui::{Look, Report};

pub(super) fn socket(report: &mut Report, key: &str, item: &Value, look: Look, width: usize) {
    report.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(
            ports::protocol(item, key).to_uppercase(),
            look.palette.accent(),
        ),
        Span::styled(
            format!("  {}", ports::endpoint(item, key)),
            look.palette.heading(),
        ),
    ]));
    report.blank();

    for (name, value) in [
        ("protocol", ports::protocol(item, key).to_string()),
        ("address", text(item, "address").unwrap_or_default()),
        (
            "port",
            number(item, "port")
                .map(|it| it.to_string())
                .unwrap_or_default(),
        ),
        ("path", text(item, "path").unwrap_or_default()),
        ("user", ports::user(item)),
        (
            "uid",
            number(item, "uid")
                .map(|it| it.to_string())
                .unwrap_or_default(),
        ),
        ("object", key.to_string()),
    ] {
        if value.is_empty() {
            continue;
        }
        named(report, look, name, &value, width);
    }
    report.blank();

    report.push(section::rule(look, "WHAT IS HOLDING IT", width));
    if item.get("owner_resolved").and_then(Value::as_bool) != Some(true) {
        for line in wrap::wrap(
            "Not resolved. The socket is there and the process behind it was out of reach. \
             That is not the same as nothing holding it. The summary screen says what the \
             collector was refused.",
            width.saturating_sub(5),
        ) {
            report.push(Line::styled(format!("   {line}"), look.palette.alarm()));
        }
        report.blank();
    } else {
        let process = item.get("process");
        let executable = process
            .and_then(|process| process.get("exe"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        named(report, look, "program", executable, width);
        if process
            .and_then(|process| process.get("exe_deleted"))
            .and_then(Value::as_bool)
            == Some(true)
        {
            for line in wrap::wrap(
                "That file has been unlinked while the process is still running.",
                width.saturating_sub(5),
            ) {
                report.push(Line::styled(format!("   {line}"), look.palette.alarm()));
            }
        }
        if let Some(line) = process
            .and_then(|process| process.get("cmdline"))
            .and_then(Value::as_str)
        {
            named(report, look, "command", line, width);
        }
        if process
            .and_then(|process| process.get("cmdline_redacted"))
            .and_then(Value::as_bool)
            == Some(true)
        {
            for line in wrap::wrap(
                "Part of this command line was hidden on this host before it was written \
                 down.",
                width.saturating_sub(5),
            ) {
                report.push(Line::styled(format!("   {line}"), look.palette.quiet()));
            }
        }
        report.blank();
    }

    silencing(report, key, look, width);
}

fn text(item: &Value, name: &str) -> Option<String> {
    item.get(name).and_then(Value::as_str).map(str::to_string)
}

fn number(item: &Value, name: &str) -> Option<u64> {
    item.get(name).and_then(Value::as_u64)
}
