use super::Surveyed;
use super::prose::comment;
use crate::Config;

pub fn schedule(survey: &[Surveyed], defaults: &Config) -> String {
    let mut out = String::new();
    for line in [
        "How often each collector reads.".to_string(),
        String::new(),
        "Each number is that collector's own default, so a removed line changes nothing and a missing key reads everything at its default. A period is a claim about how often this host is looked at: shorter costs more and catches a socket that opens and closes between readings, longer costs less and delays the news about an account.".into(),
        String::new(),
        "A name here that is not in `collectors:` above is refused when this file is read: a period for a collector that does not run.".into(),
    ] {
        out.push_str(&comment(&line));
    }

    out.push_str("schedule:\n");
    for collector in survey.iter().filter(|collector| collector.runs_here()) {
        out.push_str(&format!(
            "  {}: {}\n",
            collector.name,
            defaults.every_seconds(&collector.name)
        ));
    }
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

    fn rendered() -> String {
        let survey = vec![
            surveyed("ports", Health::Ok),
            surveyed("users", Health::Ok),
            surveyed(
                "launches",
                Health::Unavailable("auditd is not running".into()),
            ),
        ];

        schedule(&survey, &Config::default())
    }

    #[test]
    fn what_it_writes_is_a_schedule_and_it_names_only_the_collectors_that_run_here() {
        let text = rendered();

        assert!(
            text.contains("schedule:\n  ports: 30\n  users: 300\n"),
            "{text}"
        );
        assert!(
            !text.contains("  launches: "),
            "a period for a collector that cannot run here is refused when the file is read: {text}"
        );
    }

    #[test]
    fn a_generated_schedule_carries_the_periods_the_collectors_declare() {
        let path = std::env::temp_dir().join(format!(
            "vigil-generated-schedule-{}-{}.yaml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&path, rendered()).expect("writes");

        let config =
            crate::config::load(path.to_str().expect("utf-8")).expect("the daemon reads it");

        assert_eq!(config.every_seconds("ports"), 30);
        assert_eq!(config.every_seconds("users"), 300);
        assert_eq!(config.interval_seconds, None);
        let _ = std::fs::remove_file(&path);
    }
}
