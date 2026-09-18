use vigil_model::{Change, Finding, KnownKind, Severity};
use vigil_rules::{Rule, RuleContext};

use super::engine_finding::{EngineFinding, build, note};
use crate::helpers::{field_flag, field_text, parts_of};
use crate::types::Subject;

pub struct RegistryWithoutTls;

impl Rule for RegistryWithoutTls {
    fn name(&self) -> &'static str {
        "registry_without_tls"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let (engine, Subject::Registry, host) = parts_of(key)? else {
            return None;
        };
        if !field_flag(after, "insecure") {
            return None;
        }
        if before.is_some_and(|before| field_flag(before, "insecure")) {
            return None;
        }

        Some(build(
            EngineFinding {
                kind: KnownKind::ContainerRegistryInsecure,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "registry",
                title: format!(
                    "{} may pull images from {host} without TLS: whoever answers for that name \
                     on the way decides what this host runs",
                    engine.name()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![note(format!(
                    "written in {} as a {} registry",
                    field_text(after, "from").unwrap_or("a file of the engine"),
                    field_text(after, "role").unwrap_or("configured")
                ))],
            },
            ctx,
        ))
    }
}
