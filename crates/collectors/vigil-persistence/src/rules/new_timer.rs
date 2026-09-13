use vigil_model::{Change, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build, origin_evidence};
use crate::types::{Family, PersistenceView};
use vigil_rules::{Rule, RuleContext};

pub struct NewTimer;

impl Rule for NewTimer {
    fn name(&self) -> &'static str {
        "new_timer"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Timer) {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceTimerNew,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: "timer",
                title: format!(
                    "New systemd timer {} starting {} ({})",
                    view.name(),
                    view.activates(),
                    view.schedule()
                ),
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
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        NewTimer.apply(change, &mut ctx)
    }

    #[test]
    fn names_what_the_timer_starts_and_when() {
        let finding = apply(&Change::Added {
            key: "timer|certbot.timer".into(),
            after: fixture::timer("certbot.timer", &["daily"], None),
        })
        .expect("fires");

        assert_eq!(finding.finding_key, "persistence|timer|certbot.timer");
        assert!(
            finding.title.contains("certbot.service"),
            "{}",
            finding.title
        );
        assert!(finding.title.contains("daily"), "{}", finding.title);
    }

    #[test]
    fn a_service_is_somebody_elses_rule() {
        let change = Change::Added {
            key: "unit|nginx.service".into(),
            after: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
