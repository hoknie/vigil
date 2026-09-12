use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct KernelModuleLoaded;

impl Rule for KernelModuleLoaded {
    fn name(&self) -> &'static str {
        "kernel_module_loaded"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Module) {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceKernelModuleLoaded,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: "kernel_module",
                title: format!("Kernel module {} loaded", view.name()),
                before: None,
                after: Some(after.clone()),
                evidence: vec![Evidence {
                    kind: "note".into(),
                    value: format!(
                        "{} bytes of kernel memory, state {}",
                        after["size"].as_u64().unwrap_or(0),
                        after["state"].as_str().unwrap_or("?")
                    ),
                }],
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
        KernelModuleLoaded.apply(change, &mut ctx)
    }

    #[test]
    fn a_module_appearing_is_reported_under_a_key_a_person_can_suppress() {
        let finding = apply(&Change::Added {
            key: "module|diamorphine".into(),
            after: fixture::kernel_module("diamorphine", 16384),
        })
        .expect("fires");

        assert_eq!(finding.finding_key, "persistence|module|diamorphine");
        assert_eq!(finding.kind.as_str(), "persistence.kernel_module.loaded");
    }

    #[test]
    fn a_module_going_away_is_not_a_module_arriving() {
        let change = Change::Removed {
            key: "module|loop".into(),
            before: fixture::kernel_module("loop", 32768),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn the_row_saying_the_list_could_not_be_read_produces_nothing() {
        let change = Change::Added {
            key: "modules|unreadable".into(),
            after: serde_json::json!({"readable": false, "reason": "…"}),
        };

        assert!(apply(&change).is_none());
    }
}
