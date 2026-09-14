use vigil_launches::Watching;

use super::prose::{WIDTH, comment, indented_comment, wrap};
use super::{Surveyed, periods};
use crate::Config;

pub fn configuration(taken_at: &str, survey: &[Surveyed], defaults: &Config) -> String {
    let mut out = String::new();

    out.push_str(&preamble(taken_at));
    out.push('\n');
    out.push_str(&format!("state_dir: {}\n", defaults.state_dir));
    out.push_str(&format!("socket_path: {}\n", defaults.socket_path));
    out.push_str(&format!("retention_days: {}\n", defaults.retention_days));
    out.push('\n');
    out.push_str(&collectors(taken_at, survey));
    out.push('\n');
    out.push_str(&periods::schedule(survey, defaults));
    out.push('\n');
    out.push_str(REPORTERS);
    out.push('\n');
    out.push_str(SUPPRESSIONS);
    out.push('\n');
    out.push_str(&killing(defaults));
    out.push('\n');
    out.push_str(&arguments());

    out
}

fn preamble(taken_at: &str) -> String {
    let mut out = String::new();
    for line in [
        format!("vigil: the configuration of this host, written by `vigild configure` at {taken_at}."),
        String::new(),
        "This is a reading of THIS host, not a template. Every collector below was asked whether it can run here, and the answer put its name in the list or in a comment. Ask again without writing anything: `vigild configure --dry-run`.".into(),
        String::new(),
        "Every value here is also its default, so a removed line changes nothing. The one exception is the list of collectors: see the note above it.".into(),
        String::new(),
        "Under the shipped systemd unit this agent has a /tmp and a /var/tmp of its own (PrivateTmp=yes). A path under either reads as absent even when the host has a file there, so a binary run from /tmp can be reported as no longer on disk. Drop PrivateTmp with `systemctl edit vigild` to watch those two directories.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out
}

fn collectors(taken_at: &str, survey: &[Surveyed]) -> String {
    let mut out = String::new();
    for line in [
        "What is watched. ONLY WHAT IS NAMED HERE RUNS.".to_string(),
        String::new(),
        "Take a name out and that collector stops. It is named on start-up and in the console, and the reading it kept is dropped: put the name back later and the next reading is a fresh baseline.".into(),
        String::new(),
        "Remove the whole key and every collector this build has runs. An empty list watches nothing. That is supported, and it is said on start-up.".into(),
        String::new(),
        format!("Asked on this host at {taken_at}:"),
    ] {
        out.push_str(&comment(&line));
    }

    out.push_str("collectors:\n");
    for collector in survey.iter().filter(|collector| collector.runs_here()) {
        out.push_str(&indented_comment(&format!(
            "{} — {}",
            collector.state(),
            collector.subject
        )));
        if let Some(reason) = collector.reason() {
            for line in wrap(reason, WIDTH - 8) {
                out.push_str(&format!("  #     {line}\n"));
            }
        }
        out.push_str(&format!("  - {}\n", collector.name));
    }

    let absent: Vec<&Surveyed> = survey
        .iter()
        .filter(|collector| !collector.runs_here())
        .collect();
    if !absent.is_empty() {
        out.push('\n');
        for collector in absent {
            out.push_str(&indented_comment(&format!(
                "NOT ENABLED: {} cannot run on this host, so nothing watches {}:",
                collector.name, collector.subject
            )));
            for line in wrap(
                collector.reason().unwrap_or("no reason was given"),
                WIDTH - 8,
            ) {
                out.push_str(&format!("  #     {line}\n"));
            }
            out.push_str(&indented_comment(
                "Put that right and take the # off the line below.",
            ));
            out.push_str(&format!("  #  - {}\n", collector.name));
        }
    }
    out
}

const REPORTERS: &str = "\
# Where findings go. An empty list is supported: the console reads the local history, and on a
# host with no network out nothing else is needed. Uncomment one. This file is 0600 because
# a receiver here may name a file with a token in it.
#
#   - kind: ndjson
#     path: /var/log/vigil/findings.ndjson
#   - kind: syslog
#     facility: local0
reporters: []
";

const SUPPRESSIONS: &str = "\
# What this host is expected to do, and therefore what not to report. There is no learning
# window and no grace period: the first reading of a collector is a baseline and produces
# nothing, and everything after it is reported unless a line here says otherwise. Each entry
# needs a reason, in your words.
#
#   - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
#     reason: the staging api, expected on this host
suppressions: []
";

fn killing(defaults: &Config) -> String {
    let mut out = String::new();
    for line in [
        "Whether a person at the console of this host may ask the agent to close a listening socket.".to_string(),
        String::new(),
        "Off by default, and this is the only key in this file that lets the agent change anything on a host it did not set up. On, the console can ask for one of three things against the sockets a person marked there: SIGTERM to the process holding one, SIGKILL to it, or closing the socket itself and leaving the process running. The console asks the person to confirm; the agent asks nothing and does what it was told.".into(),
        String::new(),
        "Everything it does, and everything it refuses to do, is a finding of its own, so what happened is in the journal and at whatever receiver this file names. The agent will not signal pid 1 and will not signal itself.".into(),
        String::new(),
        "Nothing reaches this from the network. The console socket is 0600 and local; whoever can read it can already read every process on this host. Turning this on gives that account one more thing: it can stop them.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("killing:\n");
    out.push_str(&format!(
        "  from_the_console: {}\n",
        defaults.killing.from_the_console
    ));
    out
}

fn arguments() -> String {
    let mut out = String::new();
    for line in [
        "Whether a finding about a program launch may carry the arguments the command was given.".to_string(),
        String::new(),
        "Off by default. Off means the arguments are never assembled, so they reach neither the local history nor a receiver. On, secrets are removed first, and that removal is not complete. The shipped example in /etc/vigil says what is at stake.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("launches:\n");
    out.push_str(&format!(
        "  record_arguments: {}\n",
        Watching::default().record_arguments
    ));
    out
}

#[cfg(test)]
mod tests {
    use vigil_collect::Health;

    use super::*;

    fn surveyed(name: &str, health: Health) -> Surveyed {
        Surveyed {
            name: name.to_string(),
            subject: crate::modules::subject_of(name)
                .unwrap_or("what it watches")
                .to_string(),
            health,
        }
    }

    fn survey() -> Vec<Surveyed> {
        vec![
            surveyed("ports", Health::Ok),
            surveyed("users", Health::Ok),
            surveyed(
                "launches",
                Health::Unavailable(
                    "auditd is not running — program launches are not visible (there is no /var/log/audit/audit.log)".into(),
                ),
            ),
        ]
    }

    fn flattened(text: &str) -> String {
        text.lines()
            .map(|line| line.trim_start().trim_start_matches('#').trim())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn rendered() -> String {
        configuration("2026-09-09T13:00:00.000Z", &survey(), &Config::default())
    }

    #[test]
    fn what_it_writes_is_a_file_the_daemon_reads() {
        let path = std::env::temp_dir().join(format!(
            "vigil-generated-{}-{}.yaml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&path, rendered()).expect("writes");

        let config = crate::config::load(path.to_str().expect("utf-8"))
            .expect("the daemon reads what configure wrote");

        assert_eq!(
            config.collectors,
            Some(vec!["ports".to_string(), "users".to_string()]),
            "only what can run here is switched on"
        );
        assert_eq!(config.state_dir, Config::default().state_dir);
        assert!(config.reporters.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_collector_that_cannot_run_here_is_commented_out_with_the_reason_beside_it() {
        let text = rendered();

        assert!(text.contains("#  - launches"), "{text}");
        assert!(flattened(&text).contains("auditd is not running"), "{text}");
        assert!(flattened(&text).contains("what people run"), "{text}");
    }

    #[test]
    fn a_degraded_collector_is_switched_on_and_says_what_it_is_missing() {
        let survey = vec![surveyed(
            "launches",
            Health::Degraded("the audit rule is not loaded; run augenrules --load".into()),
        )];

        let text = configuration("2026-09-09T13:00:00.000Z", &survey, &Config::default());

        assert!(text.contains("  - launches"), "{text}");
        assert!(flattened(&text).contains("augenrules --load"), "{text}");
    }

    #[test]
    fn the_numbers_in_it_are_the_daemons_own_defaults() {
        let text = rendered();
        let defaults = Config::default();

        assert!(text.contains(&format!("retention_days: {}", defaults.retention_days)));
        assert!(text.contains(&format!("socket_path: {}", defaults.socket_path)));
        assert!(
            !text.contains("interval_seconds"),
            "one number for every collector is not a reading of this host: {text}"
        );
    }

    #[test]
    fn a_host_where_nothing_can_run_still_writes_a_file_that_reads() {
        let survey = vec![surveyed(
            "ports",
            Health::Unavailable("no /proc on this host".into()),
        )];

        let text = configuration("2026-09-09T13:00:00.000Z", &survey, &Config::default());

        assert!(text.contains("collectors:\n"), "{text}");
        assert!(text.contains("#  - ports"), "{text}");
    }

    #[test]
    fn no_line_of_a_reason_is_broken_in_the_middle_of_a_path() {
        let long = "auditd is not running — program launches are not visible (there is no /var/log/audit/audit.log, and the audit plugin has left nothing at /var/lib/vigil/audit-spool)";
        let survey = vec![surveyed("launches", Health::Unavailable(long.into()))];

        let text = configuration("2026-09-09T13:00:00.000Z", &survey, &Config::default());

        assert!(text.contains("/var/log/audit/audit.log,"), "{text}");
        assert!(text.contains("/var/lib/vigil/audit-spool)"), "{text}");
        for line in text.lines() {
            assert!(line.len() <= WIDTH + 6, "{line}");
        }
    }
}
