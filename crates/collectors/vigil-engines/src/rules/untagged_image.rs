use vigil_model::{Change, Finding, KnownKind, Severity};
use vigil_rules::{Rule, RuleContext};

use super::engine_finding::{EngineFinding, build, note};
use crate::helpers::{parts_of, untagged_image};
use crate::types::Subject;

pub struct ContainerFromAnUntaggedImage;

impl Rule for ContainerFromAnUntaggedImage {
    fn name(&self) -> &'static str {
        "container_untagged_image"
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
        let image = untagged_image(after)?;
        if before.is_some_and(|before| untagged_image(before).is_some()) {
            return None;
        }

        Some(build(
            EngineFinding {
                kind: KnownKind::ContainerImageUntaggedInUse,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: "container",
                title: format!(
                    "The {} container {name} runs an image no tag names: {image}",
                    engine.name()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![note(
                    "a tag is how a reader knows what an image is and where it came from; this \
                     one can be named only by its id, so it was built here, loaded from a file, \
                     or its tag has since moved to another image",
                )],
            },
            ctx,
        ))
    }
}
