use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::super::persistence_finding::{PersistenceFinding, build};
use crate::types::{Family, PersistenceView};
use vigil_rules::{Rule, RuleContext};

pub struct CronJobRemoved;

impl Rule for CronJobRemoved {
    fn name(&self) -> &'static str {
        "cron_job_removed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Removed { key, before } = change else {
            return None;
        };
        let view = PersistenceView::new(key, before);
        if !view.is(Family::Cron) {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceCronRemoved,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: "cron_job",
                title: format!(
                    "A scheduled job of {} is gone ({}): {}",
                    view.user(),
                    view.schedule(),
                    view.command()
                ),
                before: Some(before.clone()),
                after: None,
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
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-16T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        CronJobRemoved.apply(change, &mut ctx)
    }

    #[test]
    fn a_line_taken_out_of_a_crontab_is_reported_and_closes_the_finding_that_it_arrived() {
        let finding = apply(&Change::Removed {
            key: "cron|/etc/crontab|root|/usr/local/bin/backup".into(),
            before: fixture::cron_job("/etc/crontab", "root", "0 3 * * *", "/usr/local/bin/backup"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "persistence.cron.removed");
        assert_eq!(
            finding.finding_key, "persistence|cron|/etc/crontab|root|/usr/local/bin/backup",
            "it is keyed as the arrival was, so the journal closes that finding instead of \
             holding a job open for the life of the host"
        );
        assert!(finding.before.is_some() && finding.after.is_none());
        assert!(
            finding.title.contains("/usr/local/bin/backup"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_unit_that_went_away_is_somebody_elses_rule_or_nobodys() {
        assert!(
            apply(&Change::Removed {
                key: "unit|nginx.service".into(),
                before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
            })
            .is_none()
        );
        assert!(
            apply(&Change::Added {
                key: "cron|/etc/crontab|root|/usr/local/bin/backup".into(),
                after: fixture::cron_job(
                    "/etc/crontab",
                    "root",
                    "0 3 * * *",
                    "/usr/local/bin/backup",
                ),
            })
            .is_none(),
            "a job that appeared is the other rule's finding, and two findings about one \
             line is the noise this product dies of"
        );
    }
}
