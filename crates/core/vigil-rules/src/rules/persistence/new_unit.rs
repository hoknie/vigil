use vigil_model::{Change, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build, origin_evidence};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct NewUnit;

impl Rule for NewUnit {
    fn name(&self) -> &'static str {
        "new_unit"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Unit) {
            return None;
        }

        let severity = match view.runs_from_writable_path() {
            true => Severity::Critical,
            false => Severity::Medium,
        };

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceUnitNew,
                severity,
                rule: self.name(),
                key,
                object: "unit",
                title: match view.commands().is_empty() {
                    true => format!("New systemd unit {}", view.name()),
                    false => format!(
                        "New systemd unit {} running {} as {}",
                        view.name(),
                        view.commands(),
                        view.run_as()
                    ),
                },
                before: None,
                after: Some(after.clone()),
                evidence: origin_evidence(&view),
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
        NewUnit.apply(change, &mut ctx)
    }

    #[test]
    fn a_unit_started_out_of_a_writable_directory_outranks_a_packaged_one() {
        let packaged = apply(&Change::Added {
            key: "unit|nginx.service".into(),
            after: fixture::unit("nginx.service", "/usr/sbin/nginx -g 'daemon off;'", "root"),
        })
        .expect("fires");
        let planted = apply(&Change::Added {
            key: "unit|update.service".into(),
            after: fixture::unit("update.service", "/tmp/.x/implant", "root"),
        })
        .expect("fires");

        assert_eq!(packaged.severity, Severity::Medium);
        assert_eq!(planted.severity, Severity::Critical);
        assert_eq!(packaged.finding_key, "persistence|unit|nginx.service");
        assert!(
            packaged.title.contains("/usr/sbin/nginx"),
            "{}",
            packaged.title
        );
    }

    #[test]
    fn a_timer_is_somebody_elses_rule() {
        let change = Change::Added {
            key: "timer|certbot.timer".into(),
            after: fixture::timer("certbot.timer", &["daily"], None),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_unit_that_went_away_is_not_a_new_one() {
        let change = Change::Removed {
            key: "unit|nginx.service".into(),
            before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
