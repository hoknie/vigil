use super::kind::Kind;
use super::row::Row;
use super::showing::Showing;
use super::tree::tree;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, Startup, View};

pub const COLLECTOR: &str = "persistence";

pub fn rows<'a>(view: &'a View, showing: &Showing<'_>) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = match showing.as_a_tree() {
        true => tree(&snapshot.items)
            .into_iter()
            .map(|placed| Row {
                kind: Kind::of(&placed.key),
                item: &snapshot.items[&placed.key],
                depth: placed.depth,
                parents: placed.parents,
                key: placed.key,
            })
            .collect(),
        false => snapshot
            .items
            .iter()
            .filter(|(key, _)| Startup::holding(key) == showing.list)
            .map(|(key, item)| Row {
                key: key.clone(),
                kind: Kind::of(key),
                item,
                depth: 0,
                parents: 0,
            })
            .collect(),
    };

    rows.retain(|row| {
        showing
            .search
            .matches(&haystack::haystack(&row.key, row.item))
    });
    if !showing.as_a_tree() {
        rows.sort_by_key(|row| (!row.mark(), row.key.clone()));
    }
    rows
}

pub fn keys(view: &View, showing: &Showing<'_>) -> Vec<String> {
    rows(view, showing).into_iter().map(|row| row.key).collect()
}

pub fn objects(view: &View, list: Startup) -> usize {
    counted(view, list, false)
}

pub fn marks(view: &View, list: Startup) -> usize {
    counted(view, list, true)
}

pub fn in_the_reading(view: &View) -> usize {
    match view.reading(COLLECTOR) {
        Reading::Taken(snapshot) => snapshot.items.len(),
        _ => 0,
    }
}

fn counted(view: &View, list: Startup, marked: bool) -> usize {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return 0;
    };
    snapshot
        .items
        .keys()
        .filter(|key| Startup::holding(key) == list && Kind::of(key).mark() == marked)
        .count()
}
