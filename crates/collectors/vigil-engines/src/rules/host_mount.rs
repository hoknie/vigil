use vigil_model::{Change, Finding, KnownKind, Severity};
use vigil_rules::{Rule, RuleContext};

use super::engine_finding::{EngineFinding, build, note};
use crate::helpers::{held_of_this_host, parts_of};
use crate::types::Subject;

pub struct HeldOfThisHost;

impl Rule for HeldOfThisHost {
    fn name(&self) -> &'static str {
        "engine_host_mount"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let (engine, subject, name) = parts_of(key)?;
        let held = held_of_this_host(engine, subject, after);
        if held.is_empty() {
            return None;
        }
        if before.is_some_and(|before| held_of_this_host(engine, subject, before) == held) {
            return None;
        }

        let (object, title) = match subject {
            Subject::Volume => (
                "volume",
                format!(
                    "The {} volume {name} is {} of this host: what a container writes into it, \
                     this host reads or runs",
                    engine.name(),
                    held.join(", ")
                ),
            ),
            _ => (
                "container",
                format!(
                    "The {} container {name} mounts {} of this host: what it writes there, \
                     this host reads or runs",
                    engine.name(),
                    held.join(", ")
                ),
            ),
        };

        Some(build(
            EngineFinding {
                kind: KnownKind::ContainerVolumeHostMount,
                severity: match held.contains(&"/") {
                    true => Severity::Critical,
                    false => Severity::High,
                },
                rule: self.name(),
                key,
                object,
                title,
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![note(format!("paths of this host: {}", held.join(", ")))],
            },
            ctx,
        ))
    }
}
