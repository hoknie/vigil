use vigil_model::{Change, Finding, KnownKind, Severity};
use vigil_rules::{Rule, RuleContext};

use super::engine_finding::{EngineFinding, build, note};
use crate::helpers::{field_flag, field_list, parts_of};
use crate::types::Subject;

pub struct ContainerOnTheHostNetwork;

impl Rule for ContainerOnTheHostNetwork {
    fn name(&self) -> &'static str {
        "container_host_network"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let (engine, Subject::Container, name) = parts_of(key)? else {
            return None;
        };
        if !field_flag(after, "host_network") {
            return None;
        }
        if before.is_some_and(|before| field_flag(before, "host_network")) {
            return None;
        }

        Some(build(
            EngineFinding {
                kind: KnownKind::ContainerNetworkHostMode,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "container",
                title: format!(
                    "The {} container {name} shares the network of this host: it listens and \
                     connects as this host does, and no bridge stands between",
                    engine.name()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![note(format!(
                    "networks: {}",
                    field_list(after, "networks").join(", ")
                ))],
            },
            ctx,
        ))
    }
}
