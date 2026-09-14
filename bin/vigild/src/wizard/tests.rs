use vigil_collect::Health;

use super::prose::WIDTH;
use super::{Surveyed, configuration};
use crate::Config;

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
