use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build};
use super::resource_limits::ResourceLimits;
use crate::types::{Family, ResourceView};
use vigil_rules::{Rule, RuleContext};

pub const FINDING_KEY: &str = "resource|memory";

pub struct MemoryLow {
    limits: ResourceLimits,
}

impl MemoryLow {
    pub fn watching(limits: ResourceLimits) -> Self {
        MemoryLow { limits }
    }
}

impl Rule for MemoryLow {
    fn name(&self) -> &'static str {
        "memory_low"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ResourceView::new(key, after);
        if !now.is(Family::Memory)
            || !self
                .limits
                .below_the_memory_limit(now.available_percent_step())
        {
            return None;
        }
        if before.is_some_and(|before| {
            self.limits
                .below_the_memory_limit(ResourceView::new(key, before).available_percent_step())
        }) {
            return None;
        }

        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceMemoryLow,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                finding_key: FINDING_KEY.to_string(),
                object: "host",
                title: format!(
                    "Less than {}% of this host's memory is available",
                    self.limits.memory_available_percent
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![Evidence {
                    kind: "note".into(),
                    value: format!(
                        "at least {}% of {} bytes is available without pushing anything out; memory available is read in steps of five percent",
                        now.available_percent_step().unwrap_or(0),
                        now.total_bytes()
                    ),
                }],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn memory(total: u64, step: Option<u64>) -> serde_json::Value {
        json!({"total_bytes": total, "swap_total_bytes": null, "available_percent_step": step, "readable": true})
    }

    fn apply(before: serde_json::Value, after: serde_json::Value) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-10-31T12:00:00.000Z".into(),
            mint_event_id: &mut mint,
        };
        MemoryLow::watching(ResourceLimits::default()).apply(
            &Change::Changed {
                key: "memory|summary".into(),
                before,
                after,
            },
            &mut ctx,
        )
    }

    #[test]
    fn memory_that_fell_under_the_limit_is_reported_under_one_key() {
        let finding = apply(memory(8, Some(30)), memory(8, Some(5))).expect("fires");

        assert_eq!(finding.kind.as_str(), "resource.memory_low");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.finding_key, "resource|memory");
    }

    #[test]
    fn memory_already_under_the_limit_that_fell_further_is_not_a_second_finding() {
        assert!(apply(memory(8, Some(5)), memory(8, Some(0))).is_none());
    }

    #[test]
    fn a_change_of_total_memory_says_nothing() {
        assert!(apply(memory(8, Some(60)), memory(16, Some(60))).is_none());
    }
}
