use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use vigil_model::{Change, KnownKind, Severity};
use vigil_rules::{Batch, BatchRule, RuleContext};

use super::engine_finding::{EngineFinding, build, note};
use crate::helpers::{field_list, parts_of, silenced};
use crate::types::{Engine, Subject};

pub struct ImageTagMoved;

struct Moved<'a> {
    tag: &'a str,
    from: &'a str,
    was: &'a Value,
}

type Tagged<'a> = BTreeMap<(Engine, &'a str), (&'a str, &'a Value)>;

impl BatchRule for ImageTagMoved {
    fn name(&self) -> &'static str {
        "image_tag_moved"
    }

    fn apply(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Batch {
        let quiet = silenced(changes);
        let (lost, gained) = tags(changes, &quiet);

        let mut moved: BTreeMap<&str, (Engine, &Value, Vec<Moved<'_>>)> = BTreeMap::new();
        for ((engine, tag), (to, is)) in &gained {
            let Some((from, was)) = lost.get(&(*engine, *tag)) else {
                continue;
            };
            if from == to {
                continue;
            }
            moved
                .entry(*to)
                .or_insert_with(|| (*engine, *is, Vec::new()))
                .2
                .push(Moved { tag, from, was });
        }

        let mut batch = Batch::silent();
        for (to, (engine, is, tags)) in moved {
            batch.claimed.insert(to.to_string());
            for one in &tags {
                batch.claimed.insert(one.from.to_string());
            }
            batch.findings.push(build(
                EngineFinding {
                    kind: KnownKind::ContainerImageChanged,
                    severity: Severity::Medium,
                    rule: self.name(),
                    key: to,
                    object: "image",
                    title: format!(
                        "{} of {} now names another image",
                        tags.iter()
                            .map(|one| one.tag)
                            .collect::<Vec<&str>>()
                            .join(", "),
                        engine.name()
                    ),
                    before: tags.first().map(|one| one.was.clone()),
                    after: Some(is.clone()),
                    evidence: tags
                        .iter()
                        .map(|one| {
                            note(format!(
                                "{}: {} \u{2192} {}",
                                one.tag,
                                id_of(one.from),
                                id_of(to)
                            ))
                        })
                        .collect(),
                },
                ctx,
            ));
        }
        batch
    }
}

fn tags<'a>(changes: &'a [Change], quiet: &BTreeSet<String>) -> (Tagged<'a>, Tagged<'a>) {
    let mut lost = Tagged::new();
    let mut gained = Tagged::new();

    for change in changes {
        let key = change.key();
        let Some((engine, Subject::Image, _)) = parts_of(key) else {
            continue;
        };
        if quiet.contains(key) {
            continue;
        }
        let (before, after): (Vec<&str>, Vec<&str>) = match change {
            Change::Added { after, .. } => (Vec::new(), field_list(after, "tags")),
            Change::Removed { before, .. } => (field_list(before, "tags"), Vec::new()),
            Change::Changed { before, after, .. } => {
                (field_list(before, "tags"), field_list(after, "tags"))
            }
        };
        if let Change::Removed { before: was, .. } | Change::Changed { before: was, .. } = change {
            for tag in before.iter().filter(|tag| !after.contains(tag)) {
                lost.insert((engine, *tag), (key, was));
            }
        }
        if let Change::Added { after: is, .. } | Change::Changed { after: is, .. } = change {
            for tag in after.iter().filter(|tag| !before.contains(tag)) {
                gained.insert((engine, *tag), (key, is));
            }
        }
    }

    (lost, gained)
}

fn id_of(key: &str) -> &str {
    parts_of(key).map_or(key, |(_, _, id)| id)
}
