use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::super::persistence_finding::{PersistenceFinding, build};
use crate::types::{Family, PersistenceView};
use vigil_rules::{Rule, RuleContext};

pub struct CronJobChanged;

impl Rule for CronJobChanged {
    fn name(&self) -> &'static str {
        "cron_job_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Cron) {
            return None;
        }
        let was = PersistenceView::new(key, before);
        if was.schedule() == view.schedule() && was.user() == view.user() {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceCronChanged,
                severity: match view.runs_from_writable_path() || view.schedule() == "@reboot" {
                    true => Severity::High,
                    false => Severity::Medium,
                },
                rule: self.name(),
                key,
                object: "cron_job",
                title: format!(
                    "A scheduled job now runs as {} ({}): {}",
                    view.user(),
                    view.schedule(),
                    view.command()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "path".into(),
                        value: view.source().to_string(),
                    },
                    Evidence {
                        kind: "cmdline".into(),
                        value: format!("was: {} as {}", was.schedule(), was.user()),
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
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-16T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        CronJobChanged.apply(change, &mut ctx)
    }

    fn at(schedule: &str) -> serde_json::Value {
        fixture::cron_job("/etc/crontab", "root", schedule, "/usr/local/bin/backup")
    }

    #[test]
    fn a_job_that_now_runs_at_every_boot_is_reported_with_both_halves() {
        let finding = apply(&Change::Changed {
            key: "cron|/etc/crontab|root|/usr/local/bin/backup".into(),
            before: at("0 3 * * *"),
            after: at("@reboot"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "persistence.cron.changed");
        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("@reboot"), "{}", finding.title);
        assert!(
            finding
                .evidence
                .iter()
                .any(|one| one.value.contains("0 3 * * *")),
            "the schedule it had is the half a reader compares against: {:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_job_nothing_of_which_changed_is_not_an_event() {
        let mut after = at("0 3 * * *");
        after["command_redacted"] = serde_json::json!(true);

        assert!(
            apply(&Change::Changed {
                key: "cron|/etc/crontab|root|/usr/local/bin/backup".into(),
                before: at("0 3 * * *"),
                after,
            })
            .is_none(),
            "the schedule and the account are what a changed line can change while keeping \
             its key; a finding on anything else is a finding raised every time the agent \
             learns a new field"
        );
    }

    #[test]
    fn a_unit_whose_command_changed_is_somebody_elses_rule() {
        assert!(
            apply(&Change::Changed {
                key: "unit|nginx.service".into(),
                before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
                after: fixture::unit("nginx.service", "/tmp/.x/nginx", "root"),
            })
            .is_none()
        );
    }
}
