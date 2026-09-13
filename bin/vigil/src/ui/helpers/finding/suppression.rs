use vigil_model::Finding;

pub fn snippet(finding: &Finding) -> Vec<String> {
    entry(&finding.finding_key, Some(finding.kind.as_str()))
}

pub fn entry(key: &str, kind: Option<&str>) -> Vec<String> {
    vec![
        format!("vigil suppress add {} --reason \"...\"", quoted(key)),
        match kind {
            Some(kind) => format!("  -k {kind}"),
            None => "  -k <kind>".to_string(),
        } + " only that kind \u{b7} -u <day> until then \u{b7} -p by prefix",
        "or by hand, in the file the daemon reads at start:".to_string(),
        "  suppressions:".to_string(),
        format!("    - finding_key: {}", quoted(key)),
        "      reason: \"\"".to_string(),
    ]
}

pub fn widest(key: &str, kind: Option<&str>) -> usize {
    entry(key, kind)
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    #[test]
    fn it_names_the_command_that_writes_the_entry_as_well_as_the_entry_itself() {
        let mut finding = fixture::finding("A group gained a member", Severity::High);
        finding.finding_key = "user|group|docker".into();

        let lines = snippet(&finding);

        assert_eq!(
            lines[0], "vigil suppress add \"user|group|docker\" --reason \"...\"",
            "an operator reading a key off the screen has to type it again by hand \
             without this line"
        );
        assert!(
            lines.iter().any(|line| line.contains("suppressions:")),
            "and the file it ends up in is still spelled out, for a host managed by hand \
             or by a configuration tool: {lines:#?}"
        );
    }

    #[test]
    fn the_key_in_the_snippet_is_the_finding_key_character_for_character() {
        let mut finding = fixture::finding("A group gained a member", Severity::High);
        finding.finding_key = "user|group|docker".into();

        let lines = snippet(&finding);

        assert!(
            lines.contains(&"    - finding_key: \"user|group|docker\"".to_string()),
            "{lines:#?}"
        );
    }

    #[test]
    fn it_carries_the_reason_the_daemon_refuses_to_start_without() {
        let snippet = snippet(&fixture::finding(
            "A new listening port",
            Severity::Critical,
        ));

        assert!(
            snippet.iter().any(|line| line.contains("reason:")),
            "{snippet:#?}"
        );
    }

    #[test]
    fn the_lines_that_are_not_data_fit_an_eighty_column_terminal() {
        let finding = fixture::finding("A new listening port", Severity::Critical);

        for line in snippet(&finding) {
            if line.contains(&finding.finding_key) || line.contains(finding.kind.as_str()) {
                continue;
            }
            assert!(line.chars().count() <= 77 - 3, "{line}");
        }
    }

    #[test]
    fn a_quotation_mark_in_a_key_does_not_end_the_yaml_string_early() {
        let mut finding = fixture::finding("Something odd", Severity::Low);
        finding.finding_key = "user|sshkey|deploy|a\"b".into();

        let lines = snippet(&finding);

        assert!(
            lines.iter().any(|line| line.ends_with("a\\\"b\"")),
            "the quote is escaped, not closed: {lines:#?}"
        );
    }
}
