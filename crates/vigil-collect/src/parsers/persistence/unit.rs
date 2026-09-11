use crate::parsers::processes::redact;
use crate::parsers::processes::split_command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnitFacts {
    pub description: Option<String>,
    pub commands: Vec<String>,
    pub commands_redacted: bool,
    pub run_as: Option<String>,
    pub on_calendar: Vec<String>,
    pub on_boot: Option<String>,
    pub activates: Option<String>,
}

const DESCRIPTION_LIMIT: usize = 120;

pub fn parse_unit(text: &str) -> UnitFacts {
    let mut facts = UnitFacts::default();

    for (key, value, section) in lines(text) {
        match (section.as_str(), key.as_str()) {
            ("Unit", "Description") => {
                facts.description = Some(shorten(&value, DESCRIPTION_LIMIT));
            }
            ("Service", "ExecStart") | ("Service", "ExecStartPre") => {
                let clean = redact(&split_command(&value));
                facts.commands_redacted |= clean.redacted;
                facts.commands.push(clean.text);
            }
            ("Service", "User") => facts.run_as = Some(value),
            ("Timer", "OnCalendar") => facts.on_calendar.push(value),
            ("Timer", "OnBootSec") | ("Timer", "OnUnitActiveSec") => {
                facts.on_boot = Some(value);
            }
            ("Timer", "Unit") => facts.activates = Some(value),
            _ => {}
        }
    }

    facts
}

fn lines(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut section = String::new();
    let mut pending: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim();

        if let Some(previous) = pending.take() {
            let joined = format!("{previous} {}", line.trim_end_matches('\\').trim());
            match line.ends_with('\\') {
                true => pending = Some(joined),
                false => push(&mut out, &joined, &section),
            }
            continue;
        }

        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.to_string();
            continue;
        }
        match line.ends_with('\\') {
            true => pending = Some(line.trim_end_matches('\\').trim().to_string()),
            false => push(&mut out, line, &section),
        }
    }

    if let Some(dangling) = pending {
        push(&mut out, &dangling, &section);
    }

    out
}

fn push(out: &mut Vec<(String, String, String)>, line: &str, section: &str) {
    let Some((key, value)) = line.split_once('=') else {
        return;
    };
    out.push((
        key.trim().to_string(),
        value.trim().to_string(),
        section.to_string(),
    ));
}

fn shorten(text: &str, limit: usize) -> String {
    match text.chars().count() > limit {
        true => text.chars().take(limit).collect::<String>() + "…",
        false => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_a_service_runs_and_who_it_runs_as() {
        let facts = parse_unit(
            "[Unit]\n\
             Description=A web server\n\
             \n\
             [Service]\n\
             User=www-data\n\
             ExecStartPre=/usr/sbin/nginx -t\n\
             ExecStart=/usr/sbin/nginx -g 'daemon off;'\n\
             Restart=always\n\
             \n\
             [Install]\n\
             WantedBy=multi-user.target\n",
        );

        assert_eq!(facts.description.as_deref(), Some("A web server"));
        assert_eq!(facts.run_as.as_deref(), Some("www-data"));
        assert_eq!(
            facts.commands,
            vec![
                "/usr/sbin/nginx -t".to_string(),
                "/usr/sbin/nginx -g 'daemon off;'".to_string(),
            ]
        );
        assert!(!facts.commands_redacted);
    }

    #[test]
    fn a_password_in_an_exec_line_never_leaves_the_parser() {
        let facts = parse_unit(
            "[Service]\nExecStart=/usr/bin/mysqldump --password S3cret --all-databases\n",
        );

        assert!(facts.commands_redacted);
        assert!(
            !facts.commands[0].contains("S3cret"),
            "{:?}",
            facts.commands
        );
        assert!(facts.commands[0].contains("[redacted]"));
    }

    #[test]
    fn a_command_split_over_several_lines_is_read_whole() {
        let facts = parse_unit(
            "[Service]\n\
             ExecStart=/usr/bin/prog \\\n\
             --one \\\n\
             --two\n",
        );

        assert_eq!(
            facts.commands,
            vec!["/usr/bin/prog --one --two".to_string()]
        );
    }

    #[test]
    fn a_timer_says_when_it_fires_and_what_it_starts() {
        let facts = parse_unit(
            "[Unit]\nDescription=Renew certificates\n\
             [Timer]\nOnCalendar=daily\nUnit=certbot.service\nPersistent=true\n",
        );

        assert_eq!(facts.on_calendar, vec!["daily".to_string()]);
        assert_eq!(facts.activates.as_deref(), Some("certbot.service"));
        assert!(
            facts.commands.is_empty(),
            "a timer has no [Service] section of its own"
        );
    }

    #[test]
    fn a_setting_of_the_same_name_in_another_section_is_not_the_one_we_want() {
        let facts = parse_unit("[Install]\nUnit=something.service\n");

        assert_eq!(facts.activates, None);
    }

    #[test]
    fn a_setting_before_any_section_is_not_a_setting() {
        assert_eq!(parse_unit("ExecStart=/bin/sh\n"), UnitFacts::default());
    }
}
