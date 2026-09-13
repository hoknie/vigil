use crate::helpers::redact;
use crate::helpers::split_command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronEntry {
    pub source: String,
    pub user: String,
    pub schedule: String,
    pub command: String,
    pub command_redacted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronFormat {
    WithUser,
    ForOneUser,
}

pub fn parse_crontab(text: &str, source: &str, format: CronFormat, user: &str) -> Vec<CronEntry> {
    let mut entries = Vec::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if is_assignment(line) {
            continue;
        }

        let Some((schedule, rest)) = split_schedule(line) else {
            continue;
        };
        let (who, command) = match format {
            CronFormat::WithUser => match rest.split_once(char::is_whitespace) {
                Some((who, command)) => (who.to_string(), command.trim().to_string()),
                None => continue,
            },
            CronFormat::ForOneUser => (user.to_string(), rest.to_string()),
        };
        if command.is_empty() {
            continue;
        }

        entries.push(entry(source, &who, &schedule, &command));
    }

    entries
}

pub fn cron_script(directory: &str, path: &str, schedule: &str) -> CronEntry {
    entry(directory, "root", schedule, path)
}

fn entry(source: &str, user: &str, schedule: &str, command: &str) -> CronEntry {
    let clean = redact(&split_command(command));

    CronEntry {
        source: source.to_string(),
        user: user.to_string(),
        schedule: schedule.to_string(),
        command: clean.text,
        command_redacted: clean.redacted,
    }
}

fn is_assignment(line: &str) -> bool {
    let Some((name, _)) = line.split_once('=') else {
        return false;
    };
    let name = name.trim();
    !name.is_empty()
        && !name.contains(char::is_whitespace)
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn split_schedule(line: &str) -> Option<(String, String)> {
    if line.starts_with('@') {
        let (shorthand, rest) = line.split_once(char::is_whitespace)?;
        return Some((shorthand.to_string(), rest.trim().to_string()));
    }

    let mut fields = Vec::new();
    let mut rest = line;
    for _ in 0..5 {
        let (field, tail) = rest.trim_start().split_once(char::is_whitespace)?;
        fields.push(field);
        rest = tail;
    }
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    Some((fields.join(" "), rest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_system_crontab_line_names_the_user_it_runs_as() {
        let entries = parse_crontab(
            "SHELL=/bin/sh\n\
             PATH=/usr/local/sbin:/usr/bin\n\
             # m h dom mon dow user command\n\
             17 *\t* * *\troot\tcd / && run-parts --report /etc/cron.hourly\n",
            "/etc/crontab",
            CronFormat::WithUser,
            "root",
        );

        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0].user, "root");
        assert_eq!(entries[0].schedule, "17 * * * *");
        assert_eq!(
            entries[0].command,
            "cd / && run-parts --report /etc/cron.hourly"
        );
    }

    #[test]
    fn the_same_line_in_a_user_crontab_is_all_command() {
        let entries = parse_crontab(
            "17 * * * * cd / && run-parts /etc/cron.hourly\n",
            "/var/spool/cron/crontabs/deploy",
            CronFormat::ForOneUser,
            "deploy",
        );

        assert_eq!(entries[0].user, "deploy");
        assert_eq!(entries[0].command, "cd / && run-parts /etc/cron.hourly");
    }

    #[test]
    fn a_reboot_job_is_a_job() {
        let entries = parse_crontab(
            "@reboot /tmp/.x/implant\n",
            "/var/spool/cron/crontabs/www-data",
            CronFormat::ForOneUser,
            "www-data",
        );

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].schedule, "@reboot");
        assert_eq!(entries[0].command, "/tmp/.x/implant");
    }

    #[test]
    fn a_token_in_a_cron_command_never_leaves_the_parser() {
        let entries = parse_crontab(
            "*/5 * * * * root /usr/bin/curl --token abc123 https://example.test/ping\n",
            "/etc/cron.d/ping",
            CronFormat::WithUser,
            "root",
        );

        assert!(entries[0].command_redacted);
        assert!(!entries[0].command.contains("abc123"), "{entries:?}");
    }

    #[test]
    fn settings_and_comments_are_not_jobs() {
        let entries = parse_crontab(
            "MAILTO=\n\
             SHELL=/bin/bash\n\
             # 0 0 * * * root /bin/true\n\
             \n",
            "/etc/crontab",
            CronFormat::WithUser,
            "root",
        );

        assert!(entries.is_empty(), "{entries:?}");
    }

    #[test]
    fn a_line_this_build_does_not_understand_is_skipped_rather_than_guessed_at() {
        let entries = parse_crontab(
            "not a crontab line at all\n",
            "/etc/crontab",
            CronFormat::WithUser,
            "root",
        );

        assert!(entries.is_empty());
    }

    #[test]
    fn a_script_in_a_drop_in_directory_is_the_same_three_facts() {
        let job = cron_script("/etc/cron.daily", "/etc/cron.daily/logrotate", "@daily");

        assert_eq!(job.user, "root");
        assert_eq!(job.schedule, "@daily");
        assert_eq!(job.command, "/etc/cron.daily/logrotate");
    }
}
