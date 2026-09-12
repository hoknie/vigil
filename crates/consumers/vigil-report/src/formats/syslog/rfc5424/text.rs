use serde_json::Value;

const SD_VALUE_CAP: usize = 96;
const TIMESTAMP_CAP: usize = 64;

const ELLIPSIS: &str = "…";

pub(super) fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

pub(super) fn compact(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "?".to_string())
}

pub(super) fn printable(text: &str) -> String {
    text.chars()
        .map(|c| match c.is_control() {
            true => ' ',
            false => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub(super) fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        if matches!(c, '"' | '\\' | ']') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

pub(super) fn sd_value(raw: &str) -> String {
    let (value, _) = clamp(&escape(&printable(raw)), SD_VALUE_CAP);
    value
}

pub(super) fn clamp(text: &str, cap: usize) -> (String, usize) {
    if text.len() <= cap {
        return (text.to_string(), 0);
    }
    let mut end = cap.saturating_sub(ELLIPSIS.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    while end > 0 && text.as_bytes()[end - 1] == b'\\' {
        end -= 1;
    }
    (format!("{}{ELLIPSIS}", &text[..end]), text.len() - end)
}

pub(super) fn ascii_field(text: &str, cap: usize) -> String {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_graphic())
        .take(cap)
        .collect();
    match cleaned.is_empty() {
        true => "-".to_string(),
        false => cleaned,
    }
}

pub(super) fn timestamp(text: &str) -> Option<String> {
    let candidate: String = text.trim().chars().take(TIMESTAMP_CAP).collect();
    match candidate.len() >= 20 && candidate.chars().all(|c| c.is_ascii_graphic()) {
        true => Some(candidate),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{finding, format, host};
    use vigil_model::Evidence;

    #[test]
    fn a_newline_in_a_command_line_cannot_forge_a_second_record() {
        let mut hostile = finding();
        hostile.evidence = vec![Evidence {
            kind: "cmdline".into(),
            value: "sh -c x\n<13>1 2026-09-09T12:00:00.000Z web-03 sshd - - - all clear".into(),
        }];

        let line = format().line(&hostile, &host(), "");

        assert!(!line.contains('\n'), "{line}");
        assert!(!line.contains('\r'), "{line}");
        assert_eq!(
            line.lines().count(),
            1,
            "a value the host chose must not be able to become a record: {line}"
        );
        assert!(line.starts_with("<163>1 "), "{line}");
        assert!(line.contains(r#"cmdline="sh -c x <13>1 "#), "{line}");
    }
}
