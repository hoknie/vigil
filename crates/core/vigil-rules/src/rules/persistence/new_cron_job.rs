use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct NewCronJob;

impl Rule for NewCronJob {
    fn name(&self) -> &'static str {
        "new_cron_job"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Cron) {
            return None;
        }

        let severity = match (view.runs_from_writable_path(), view.schedule() == "@reboot") {
            (true, _) => Severity::Critical,
            (false, true) => Severity::High,
            (false, false) => Severity::Medium,
        };

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceCronNew,
                severity,
                rule: self.name(),
                key,
                object: "cron_job",
                title: format!(
                    "New scheduled job as {} ({}): {}",
                    view.user(),
                    view.schedule(),
                    view.command()
                ),
                before: None,
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "path".into(),
                        value: view.source().to_string(),
                    },
                    Evidence {
                        kind: "cmdline".into(),
                        value: view.command().to_string(),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        NewCronJob.apply(change, &mut ctx)
    }

    #[test]
    fn a_reboot_job_out_of_a_writable_directory_is_the_loudest_of_the_three() {
        let backup = apply(&Change::Added {
            key: "cron|/etc/crontab|root|/usr/local/bin/backup".into(),
            after: fixture::cron_job("/etc/crontab", "root", "0 3 * * *", "/usr/local/bin/backup"),
        })
        .expect("fires");
        let at_boot = apply(&Change::Added {
            key: "cron|/var/spool/cron/crontabs/root|root|/usr/local/bin/agent".into(),
            after: fixture::cron_job(
                "/var/spool/cron/crontabs/root",
                "root",
                "@reboot",
                "/usr/local/bin/agent",
            ),
        })
        .expect("fires");
        let implant = apply(&Change::Added {
            key: "cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant".into(),
            after: fixture::cron_job(
                "/var/spool/cron/crontabs/www-data",
                "www-data",
                "@reboot",
                "/tmp/.x/implant",
            ),
        })
        .expect("fires");

        assert_eq!(backup.severity, Severity::Medium);
        assert_eq!(at_boot.severity, Severity::High);
        assert_eq!(implant.severity, Severity::Critical);
        assert_eq!(
            backup.finding_key,
            "persistence|cron|/etc/crontab|root|/usr/local/bin/backup"
        );
    }

    #[test]
    fn a_job_whose_secret_was_hidden_says_which_field_is_hidden() {
        let mut job = fixture::cron_job(
            "/etc/cron.d/ping",
            "root",
            "*/5 * * * *",
            "/usr/bin/curl --token [redacted] https://x",
        );
        job["command_redacted"] = serde_json::json!(true);

        let finding = apply(&Change::Added {
            key: "cron|/etc/cron.d/ping|root|x".into(),
            after: job,
        })
        .expect("fires");

        assert_eq!(finding.redacted, vec!["/after/command".to_string()]);
    }

    #[test]
    fn a_unit_is_somebody_elses_rule() {
        let change = Change::Added {
            key: "unit|nginx.service".into(),
            after: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
