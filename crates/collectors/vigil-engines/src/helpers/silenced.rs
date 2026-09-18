use std::collections::BTreeSet;

use vigil_model::Change;

use super::row::parts_of;
use crate::types::{Engine, Standing, Subject};

pub fn silenced(changes: &[Change]) -> BTreeSet<String> {
    let mut quiet = BTreeSet::new();

    for engine in Engine::ALL {
        let Some((was, is)) = standings(changes, engine) else {
            continue;
        };
        for change in changes {
            let Some((of, subject, _)) = parts_of(change.key()) else {
                continue;
            };
            if of != engine || subject == Subject::Engine {
                continue;
            }
            let unknown_on_one_side = match change {
                Change::Removed { .. } => is.silent_on(subject),
                Change::Added { .. } => was.silent_on(subject),
                Change::Changed { .. } => false,
            };
            if unknown_on_one_side {
                quiet.insert(change.key().to_string());
            }
        }
    }

    quiet
}

fn standings(changes: &[Change], engine: Engine) -> Option<(Standing, Standing)> {
    let key = format!(
        "{}|{}|{}",
        engine.name(),
        Subject::Engine.as_str(),
        engine.name()
    );
    let change = changes.iter().find(|change| change.key() == key)?;

    Some(match change {
        Change::Added { after, .. } => (Standing::of(None), Standing::of(Some(after))),
        Change::Removed { before, .. } => (Standing::of(Some(before)), Standing::of(None)),
        Change::Changed { before, after, .. } => {
            (Standing::of(Some(before)), Standing::of(Some(after)))
        }
    })
}
