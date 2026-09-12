use serde_json::Value;

use super::fields::sort_key;
use super::kind::Kind;
use super::row::Row;
use super::showing::Showing;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, View};

pub const COLLECTOR: &str = "firewall";

pub fn rows<'a>(view: &'a View, showing: &Showing<'_>) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = snapshot
        .items
        .iter()
        .filter_map(|(key, item)| {
            Kind::of(key).map(|kind| Row {
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

pub fn summary(view: &View) -> Option<&Value> {
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return None;
    };
    snapshot
        .items
        .iter()
        .find(|(key, _)| Kind::of(key) == Some(Kind::Ruleset))
        .map(|(_, item)| item)
}

pub fn in_the_reading(view: &View) -> usize {
    match view.reading(COLLECTOR) {
        Reading::Taken(snapshot) => snapshot.items.len(),
        _ => 0,
    }
}
