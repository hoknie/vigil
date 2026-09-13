use serde_json::Value;
use vigil_view::{Cell, Column, Width};

use super::fields::{chains, hook, kind_of_chain, policy, priority, rules, what};
use crate::types::Kind;

pub(super) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("KIND", Width::Fixed(7)),
            Column::new("WHAT", Width::Share(1)),
            Column::new("TYPE", Width::Fixed(8)),
            Column::new("HOOK", Width::Fixed(11)),
            Column::new("PRIORITY", Width::Fixed(8)),
            Column::new("POLICY", Width::Fixed(6)),
            Column::new("CHAINS", Width::Fixed(6)),
            Column::new("RULES", Width::Fixed(5)),
        ],
        false => vec![
            Column::new("KIND", Width::Fixed(7)),
            Column::new("WHAT", Width::Share(1)),
            Column::new("HOOK", Width::Fixed(11)),
            Column::new("PRIORITY", Width::Fixed(8)),
            Column::new("POLICY", Width::Fixed(6)),
            Column::new("RULES", Width::Fixed(5)),
        ],
    }
}

pub(super) fn cells(key: &str, item: &Value, wide: bool) -> Vec<Cell> {
    let kind = Kind::of(key).map_or("row", |kind| kind.name());
    let mut cells = vec![Cell::plain(kind), Cell::plain(what(key, item))];
    if wide {
        cells.push(Cell::plain(kind_of_chain(key, item)));
    }
    cells.push(Cell::plain(hook(key, item)));
    cells.push(Cell::plain(priority(key, item)));
    cells.push(Cell::plain(policy(key, item)));
    if wide {
        cells.push(Cell::plain(chains(item)));
    }
    cells.push(Cell::plain(rules(item)));

    cells
}
