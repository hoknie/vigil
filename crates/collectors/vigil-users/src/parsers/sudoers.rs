use vigil_collect::redact;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SudoGrant {
    pub who: String,
    pub source: String,
    pub spec: String,
    pub spec_redacted: bool,
    pub nopasswd: bool,
    pub all_commands: bool,
}

impl SudoGrant {
    pub fn is_group(&self) -> bool {
        self.who.starts_with('%')
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sudoers {
    pub grants: Vec<SudoGrant>,
    pub unparsed: usize,
}

pub fn parse_sudoers(text: &str, source: &str) -> Sudoers {
    let mut sudoers = Sudoers::default();

    for line in join_continuations(text) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('@') {
            continue;
        }
        if line.starts_with("Defaults") {
            continue;
        }
        if is_alias(line) {
            sudoers.unparsed += 1;
            continue;
        }

        let Some((who, spec)) = line.split_once(char::is_whitespace) else {
            sudoers.unparsed += 1;
            continue;
        };
        let spec = spec.trim();
        if !spec.contains('=') {
            sudoers.unparsed += 1;
            continue;
        }

        let upper = spec.to_ascii_uppercase();
        let clean = redact(
            &spec
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>(),
        );
        sudoers.grants.push(SudoGrant {
            who: who.to_string(),
            source: source.to_string(),
            spec: clean.text,
            spec_redacted: clean.redacted,
            nopasswd: upper.contains("NOPASSWD"),
            all_commands: commands_of(&upper) == "ALL",
        });
    }

    sudoers
}

const TAGS: &[&str] = &[
    "NOPASSWD",
    "PASSWD",
    "NOEXEC",
    "EXEC",
    "SETENV",
    "NOSETENV",
    "LOG_INPUT",
    "NOLOG_INPUT",
    "LOG_OUTPUT",
    "NOLOG_OUTPUT",
    "MAIL",
    "NOMAIL",
    "FOLLOW",
    "NOFOLLOW",
];

fn commands_of(upper_spec: &str) -> &str {
    let Some((_, rest)) = upper_spec.split_once('=') else {
        return upper_spec.trim();
    };
    let mut rest = rest.trim();

    if let Some(stripped) = rest.strip_prefix('(')
        && let Some(close) = stripped.find(')')
    {
        rest = stripped[close + 1..].trim();
    }

    while let Some((head, tail)) = rest.split_once(':') {
        if !TAGS.contains(&head.trim()) {
            break;
        }
        rest = tail.trim();
    }

    rest
}

fn is_alias(line: &str) -> bool {
    ["User_Alias", "Runas_Alias", "Host_Alias", "Cmnd_Alias"]
        .iter()
        .any(|alias| line.starts_with(alias))
}

fn join_continuations(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending = String::new();

    for raw in text.lines() {
        match raw.strip_suffix('\\') {
            Some(head) => {
                pending.push_str(head);
                pending.push(' ');
            }
            None => {
                pending.push_str(raw);
                lines.push(std::mem::take(&mut pending));
            }
        }
    }
    if !pending.is_empty() {
        lines.push(pending);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUDOERS: &str = r#"
# /etc/sudoers
Defaults	env_reset
Defaults	secure_path="/usr/local/sbin:/usr/bin"

User_Alias FULLTIMERS = alice, bob

root	ALL=(ALL:ALL) ALL
%sudo	ALL=(ALL:ALL) ALL
deploy	ALL=(ALL) NOPASSWD: ALL
backup	ALL=(root) NOPASSWD: /usr/bin/rsync
monitor ALL=(ALL) \
	/usr/bin/systemctl status *
"#;

    #[test]
    fn reads_who_may_become_somebody_else_and_keeps_the_line_a_person_can_grep_for() {
        let sudoers = parse_sudoers(SUDOERS, "/etc/sudoers");

        let who: Vec<&str> = sudoers
            .grants
            .iter()
            .map(|grant| grant.who.as_str())
            .collect();
        assert_eq!(who, ["root", "%sudo", "deploy", "backup", "monitor"]);
        assert_eq!(sudoers.grants[2].spec, "ALL=(ALL) NOPASSWD: ALL");
        assert!(!sudoers.grants[2].spec_redacted);
        assert_eq!(sudoers.grants[2].source, "/etc/sudoers");
    }

    #[test]
    fn tells_a_blanket_grant_apart_from_one_command_without_a_password() {
        let sudoers = parse_sudoers(SUDOERS, "/etc/sudoers");
        let grant = |who: &str| {
            sudoers
                .grants
                .iter()
                .find(|grant| grant.who == who)
                .expect("present")
                .clone()
        };

        assert!(grant("deploy").nopasswd && grant("deploy").all_commands);
        assert!(grant("backup").nopasswd && !grant("backup").all_commands);
        assert!(!grant("root").nopasswd && grant("root").all_commands);
        assert!(grant("%sudo").is_group() && !grant("deploy").is_group());
    }

    #[test]
    fn a_rule_split_over_two_lines_is_one_grant_not_two_pieces_of_rubbish() {
        let sudoers = parse_sudoers(SUDOERS, "/etc/sudoers");
        let monitor = sudoers
            .grants
            .iter()
            .find(|grant| grant.who == "monitor")
            .expect("present");

        assert!(monitor.spec.contains("systemctl status"), "{monitor:?}");
        assert!(!monitor.all_commands);
    }

    #[test]
    fn a_credential_glued_into_a_command_spec_is_hidden_before_it_is_stored() {
        let sudoers = parse_sudoers(
            "backup ALL=(root) /usr/bin/mysql -pS3cret\n",
            "/etc/sudoers",
        );

        assert!(sudoers.grants[0].spec_redacted);
        assert!(
            !sudoers.grants[0].spec.contains("S3cret"),
            "{:?}",
            sudoers.grants[0]
        );
        assert!(sudoers.grants[0].spec.contains("[redacted]"));
    }

    #[test]
    fn an_alias_is_counted_rather_than_dropped_where_nobody_would_see_it() {
        let sudoers = parse_sudoers(SUDOERS, "/etc/sudoers");

        assert_eq!(
            sudoers.unparsed, 1,
            "a grant this parser cannot follow must still be visible as one it could not read"
        );
    }
}
