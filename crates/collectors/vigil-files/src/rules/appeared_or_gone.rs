use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use vigil_model::{Change, Evidence, KnownKind, Severity};
use vigil_rules::{Batch, BatchRule, RuleContext};

use super::file_finding::{FileFinding, build, standing};
use crate::types::{Family, FileView};

pub struct FileAppearedOrGone;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Way {
    Arrived,
    Left,
}

struct Moved<'a> {
    key: &'a str,
    value: &'a Value,
    way: Way,
}

impl BatchRule for FileAppearedOrGone {
    fn name(&self) -> &'static str {
        "file_appeared_or_gone"
    }

    fn apply(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Batch {
        let mut walks: BTreeMap<&str, &Change> = BTreeMap::new();
        let mut moved: Vec<Moved<'_>> = Vec::new();

        for change in changes {
            match change {
                Change::Added { key, after } if walked(key, after) => moved.push(Moved {
                    key,
                    value: after,
                    way: Way::Arrived,
                }),
                Change::Removed { key, before } if walked(key, before) => moved.push(Moved {
                    key,
                    value: before,
                    way: Way::Left,
                }),
                other if Family::of(other.key()) == Some(Family::Walk) => {
                    walks.insert(&other.key()["walk|".len()..], other);
                }
                _ => {}
            }
        }

        let mut batch = Batch::silent();
        batch
            .claimed
            .extend(walks.keys().map(|entry| format!("walk|{entry}")));
        batch
            .claimed
            .extend(moved.iter().map(|one| one.key.to_string()));

        let folders: BTreeSet<(&str, Way)> = moved
            .iter()
            .filter(|one| FileView::new(one.key, one.value).is_a_directory())
            .map(|one| (FileView::new(one.key, one.value).path(), one.way))
            .collect();
        let mut under: BTreeMap<(&str, Way), usize> = BTreeMap::new();
        let mut said: Vec<&Moved<'_>> = Vec::new();

        for one in &moved {
            if !said_about(one, &walks) {
                continue;
            }
            match folded(FileView::new(one.key, one.value).path(), one.way, &folders) {
                Some(folder) => *under.entry((folder, one.way)).or_default() += 1,
                None => said.push(one),
            }
        }

        for one in said {
            let view = FileView::new(one.key, one.value);
            let beneath = under.get(&(view.path(), one.way)).copied().unwrap_or(0);
            batch.findings.push(finding(self.name(), one, beneath, ctx));
        }
        batch
    }
}

fn walked(key: &str, value: &Value) -> bool {
    let view = FileView::new(key, value);
    view.is(Family::File) && view.found_by().is_some()
}

fn said_about(one: &Moved<'_>, walks: &BTreeMap<&str, &Change>) -> bool {
    let view = FileView::new(one.key, one.value);
    if !view.complete() {
        return false;
    }
    let Some(by) = view.found_by() else {
        return false;
    };

    match walks.get(by) {
        None => true,
        Some(Change::Changed { key, before, after }) => {
            let was = FileView::new(key, before);
            let now = FileView::new(key, after);
            was.complete() && now.complete() && was.not_entered() == now.not_entered()
        }
        Some(_) => false,
    }
}

fn folded<'a>(path: &'a str, way: Way, folders: &BTreeSet<(&'a str, Way)>) -> Option<&'a str> {
    let mut at = path;
    while let Some(cut) = at.rfind('/') {
        at = &path[..cut];
        if at.is_empty() {
            return None;
        }
        if folders.contains(&(at, way)) {
            return Some(at);
        }
    }
    None
}

fn finding(
    rule: &'static str,
    one: &Moved<'_>,
    beneath: usize,
    ctx: &mut RuleContext<'_>,
) -> vigil_model::Finding {
    let view = FileView::new(one.key, one.value);
    let by = view.found_by().unwrap_or("the watch list");
    let (title, before, after) = match one.way {
        Way::Arrived => (
            format!(
                "{} is on this host, and at the reading before it was not: it appeared under {by}",
                view.path()
            ),
            None,
            Some(one.value.clone()),
        ),
        Way::Left => (
            format!(
                "{} is gone from {by}, and this host was watching it",
                view.path()
            ),
            Some(one.value.clone()),
            None,
        ),
    };

    let mut evidence = vec![standing(&view)];
    if beneath > 0 {
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!(
                "{beneath} more path(s) under it {} with it, and each is told in this finding \
                 rather than in one of its own",
                match one.way {
                    Way::Arrived => "arrived",
                    Way::Left => "went",
                }
            ),
        });
    }

    build(
        FileFinding {
            kind: KnownKind::FileChanged,
            severity: Severity::Medium,
            rule,
            key: one.key,
            object: match view.is_a_directory() {
                true => "directory",
                false => "file",
            },
            title,
            before,
            after,
            evidence,
        },
        ctx,
    )
}
