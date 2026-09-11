use super::kind::Kind;
use super::row::Row;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, Search, Startup, View};

pub const COLLECTOR: &str = "persistence";

pub fn rows<'a>(view: &'a View, list: Startup, search: &Search) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = snapshot
        .items
        .iter()
        .filter(|(key, _)| Startup::holding(key) == list)
        .filter(|(key, item)| search.matches(&haystack::haystack(key, item)))
        .map(|(key, item)| Row {
            key: key.clone(),
            kind: Kind::of(key),
            item,
        })
        .collect();

    rows.sort_by_key(|row| (!row.mark(), row.key.clone()));
    rows
}

pub fn keys(view: &View, list: Startup, search: &Search) -> Vec<String> {
    rows(view, list, search)
        .into_iter()
        .map(|row| row.key)
        .collect()
}

pub fn objects(view: &View, list: Startup) -> usize {
    rows(view, list, &Search::default())
        .into_iter()
        .filter(|row| !row.mark())
        .count()
}

pub fn in_the_reading(view: &View) -> usize {
    match view.reading(COLLECTOR) {
        Reading::Taken(snapshot) => snapshot.items.len(),
        _ => 0,
    }
}
