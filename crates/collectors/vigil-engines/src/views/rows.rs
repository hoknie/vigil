use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Facet, Index, RowKey, Showing, haystack};

use super::cells::{Row, ordered};
use super::columns::every;
use super::facts::{of_the_list, warnings};
use crate::helpers::project_of;
use crate::types::{Engine, List};

pub(super) const PROJECT: &str = "project";

struct Listed<'a> {
    rank: u8,
    key: &'a String,
    item: &'a Value,
}

pub(super) fn rows(
    reading: &Snapshot,
    engine: Engine,
    list: List,
    showing: &Showing<'_>,
) -> Vec<RowKey> {
    let mut rows: Vec<Listed<'_>> = listed(reading, engine, list)
        .into_iter()
        .filter(|one| showing.matches(one.key, one.item))
        .filter(|one| chosen(list, one.item, showing))
        .collect();

    let columns = every(list).len();
    let sorting = showing.sorting;
    if !sorting.as_read() && sorting.by <= columns {
        let keyed: Vec<(u8, String)> = rows
            .iter()
            .map(|one| {
                (
                    one.rank,
                    sort_key(reading, engine, list, one, sorting.by - 1),
                )
            })
            .collect();
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|left, right| {
            let by = keyed[*left].1.cmp(&keyed[*right].1);
            let by = match sorting.descending {
                true => by.reverse(),
                false => by,
            };
            keyed[*left].0.cmp(&keyed[*right].0).then(by)
        });
        let mut sorted: Vec<Option<Listed<'_>>> = rows.into_iter().map(Some).collect();
        rows = order
            .into_iter()
            .filter_map(|at| sorted[at].take())
            .collect();
    }

    rows.into_iter().map(|one| row_key(&one)).collect()
}

pub(super) fn index(reading: &Snapshot, engine: Engine, list: List) -> Index {
    let columns = every(list).len();
    let mut index = Index::new(columns);

    for one in listed(reading, engine, list) {
        index.push(
            row_key(&one),
            &haystack(one.key, one.item),
            one.rank,
            (0..columns)
                .map(|at| sort_key(reading, engine, list, &one, at))
                .collect(),
        );
        if list.faceted() {
            index.faceted(facets_of(one.item));
        }
    }
    index
}

pub(super) fn facets_of(item: &Value) -> Vec<Facet> {
    project_of(item)
        .map(|project| vec![Facet::new(PROJECT, project)])
        .unwrap_or_default()
}

fn chosen(list: List, item: &Value, showing: &Showing<'_>) -> bool {
    if !list.faceted() {
        return true;
    }
    let carried = facets_of(item);
    showing
        .only
        .iter()
        .enumerate()
        .filter(|(place, facet)| {
            !showing.only[..*place]
                .iter()
                .any(|before| before.name == facet.name)
        })
        .all(|(_, facet)| carried.contains(facet))
}

fn listed(reading: &Snapshot, engine: Engine, list: List) -> Vec<Listed<'_>> {
    let mut listed: Vec<Listed<'_>> = of_the_list(reading, engine, list)
        .map(|(key, item)| Listed {
            rank: u8::from(warnings(engine, list, item).is_empty()),
            key,
            item,
        })
        .collect();
    listed.sort_by(|left, right| (left.rank, left.key).cmp(&(right.rank, right.key)));
    listed
}

fn sort_key(
    reading: &Snapshot,
    engine: Engine,
    list: List,
    one: &Listed<'_>,
    column: usize,
) -> String {
    ordered(
        &Row {
            engine,
            list,
            key: one.key,
            item: one.item,
            reading,
        },
        column,
    )
}

fn row_key(one: &Listed<'_>) -> RowKey {
    RowKey::of(one.key.as_str())
}
