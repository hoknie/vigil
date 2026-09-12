use super::fields::sort_key;
use super::kind::Kind;
use super::row::Row;
use super::showing::Showing;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, View};

pub const COLLECTOR: &str = "containers";

pub fn rows<'a>(view: &'a View, showing: &Showing<'_>) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = snapshot
        .items
        .iter()
        .filter_map(|(key, item)| {
            Kind::of(key)
                .filter(|kind| *kind == Kind::Container)
                .map(|kind| Row {
                    key: key.clone(),
                    kind,
                    item,
                })
        })
        .filter(|row| {
            showing
                .search
                .matches(&haystack::haystack(&row.key, row.item))
        })
        .collect();

    rows.sort_by_key(sort_key);
    super::sorting::sort(&mut rows, showing.sorting);
    rows
}

pub fn identities(view: &View, showing: &Showing<'_>) -> Vec<String> {
    rows(view, showing)
        .iter()
        .map(super::fields::identity)
        .collect()
}

pub fn in_the_reading(view: &View) -> usize {
    match view.reading(COLLECTOR) {
        Reading::Taken(snapshot) => snapshot
            .items
            .keys()
            .filter(|key| Kind::of(key) == Some(Kind::Container))
            .count(),
        _ => 0,
    }
}

pub fn sockets(view: &View) -> Vec<(String, String)> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return Vec::new();
    };

    snapshot
        .items
        .iter()
        .filter(|(key, _)| Kind::of(key) == Some(Kind::Socket))
        .map(|(_, item)| {
            (
                super::fields::text(item, "path").to_string(),
                super::fields::text(item, "mode").to_string(),
            )
        })
        .collect()
}
