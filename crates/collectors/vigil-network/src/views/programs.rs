use std::collections::{BTreeMap, BTreeSet};

use vigil_model::Snapshot;
use vigil_view::{Showing, basename};

use super::fields::holder;
use super::flat::passing;

pub(super) const HEADING: &str = "program|";

pub(super) const UNRESOLVED: &str = "unresolved";

pub(super) type Gathered<'a> = (Vec<(String, Vec<&'a String>)>, Vec<&'a String>);

pub(super) struct Programs<'a> {
    by_name: BTreeMap<&'a str, BTreeSet<&'a str>>,
}

impl<'a> Programs<'a> {
    pub(super) fn of(
        reading: &'a Snapshot,
        showing: &Showing<'_>,
        gathered: &'a Gathered<'a>,
    ) -> Programs<'a> {
        let mut by_name: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut note = |path: &'a str| {
            by_name.entry(basename(path)).or_default().insert(path);
        };
        match showing.narrowed() {
            false => gathered.0.iter().for_each(|(path, _)| note(path)),
            true => reading.items.values().filter_map(holder).for_each(note),
        }
        Programs { by_name }
    }

    pub(super) fn whole(reading: &'a Snapshot) -> Programs<'a> {
        let mut by_name: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for path in reading.items.values().filter_map(holder) {
            by_name.entry(basename(path)).or_default().insert(path);
        }
        Programs { by_name }
    }

    pub(super) fn name(&self, path: &str) -> String {
        let shared = self
            .by_name
            .get(basename(path))
            .is_some_and(|paths| paths.len() > 1);
        shown_name(path, shared)
    }
}

pub(super) fn gathered<'a>(reading: &'a Snapshot, showing: &Showing<'_>) -> Gathered<'a> {
    let mut programs: Vec<(String, Vec<&'a String>)> = Vec::new();
    let mut unresolved: Vec<&'a String> = Vec::new();

    for (key, item) in passing(reading, showing) {
        match holder(item) {
            None => unresolved.push(key),
            Some(path) => match programs.iter_mut().find(|(known, _)| known == path) {
                Some((_, sockets)) => sockets.push(key),
                None => programs.push((path.to_string(), vec![key])),
            },
        }
    }
    programs.sort_by(|left, right| left.0.cmp(&right.0));

    (programs, unresolved)
}

pub(super) fn counted(reading: &Snapshot, heading: &str) -> usize {
    let path = heading.strip_prefix(HEADING);
    reading
        .items
        .values()
        .filter(|item| holder(item) == path)
        .count()
}

pub(super) fn named_from_the_reading(reading: &Snapshot, path: &str) -> String {
    let name = basename(path);
    let mut seen: Vec<&str> = reading
        .items
        .values()
        .filter_map(holder)
        .filter(|other| basename(other) == name)
        .collect();
    seen.sort_unstable();
    seen.dedup();
    shown_name(path, seen.len() > 1)
}

fn shown_name(path: &str, shared: bool) -> String {
    match shared {
        true => path.to_string(),
        false => basename(path).to_string(),
    }
}
