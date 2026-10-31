use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build};
use super::resource_limits::ResourceLimits;
use crate::types::{Family, ResourceView};
use vigil_rules::{Rule, RuleContext};

pub const FINDING_KEY: &str = "resource|cpu";

pub struct CpuSaturated {
    limits: ResourceLimits,
}

impl CpuSaturated {
    pub fn watching(limits: ResourceLimits) -> Self {
        CpuSaturated { limits }
    }
}

impl Rule for CpuSaturated {
    fn name(&self) -> &'static str {
        "cpu_saturated"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ResourceView::new(key, after);
        if !now.is(Family::Processors) || !self.limits.at_the_cpu_limit(now.busy_percent_step()) {
            return None;
        }
        if before.is_some_and(|before| {
            self.limits
                .at_the_cpu_limit(ResourceView::new(key, before).busy_percent_step())
        }) {
            return None;
        }

        let step = now.busy_percent_step().unwrap_or(0);
        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceCpuSaturated,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                finding_key: FINDING_KEY.to_string(),
                object: "host",
                title: format!(
                    "The processors of this host were {step}% busy or more over the last 15 minutes"
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "note".into(),
                        value: format!(
                            "{} processor(s), at least {step}% busy on average over the last {} seconds, against a limit of {}%",
                            now.processors(),
                            now.window_seconds(),
                            self.limits.cpu_busy_percent
                        ),
                    },
                    Evidence {
                        kind: "note".into(),
                        value: "busyness is read in steps of ten percent averaged over fifteen minutes, so a build or an update of a few minutes does not reach it and a program that keeps the processors busy for hours does".into(),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn cpu(step: Option<u64>) -> serde_json::Value {
        json!({"processors": 4, "busy_percent_step": step, "window_seconds": 900, "readable": true})
    }

    fn apply(before: Option<u64>, after: Option<u64>) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-10-31T12:00:00.000Z".into(),
            mint_event_id: &mut mint,
        };
        CpuSaturated::watching(ResourceLimits::default()).apply(
            &Change::Changed {
                key: "cpu|summary".into(),
                before: cpu(before),
                after: cpu(after),
            },
            &mut ctx,
        )
    }

    #[test]
    fn a_host_that_became_busy_for_fifteen_minutes_is_reported_once() {
        let finding = apply(Some(10), Some(90)).expect("fires");

        assert_eq!(finding.kind.as_str(), "resource.cpu_saturated");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.finding_key, "resource|cpu");
        assert!(finding.title.contains("90% busy"), "{}", finding.title);
    }

    #[test]
    fn a_host_that_stays_busy_is_not_reported_again() {
        assert!(apply(Some(90), Some(100)).is_none());
    }

    #[test]
    fn a_window_that_filled_on_a_busy_host_is_reported() {
        assert!(
            apply(None, Some(100)).is_some(),
            "a host busy since the agent started is said once the window is full"
        );
    }

    #[test]
    fn a_step_that_became_unknown_says_nothing() {
        assert!(apply(Some(90), None).is_none());
        assert!(apply(Some(10), None).is_none());
    }
}
