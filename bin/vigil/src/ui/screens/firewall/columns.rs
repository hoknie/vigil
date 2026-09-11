use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::{chains, hook, kind_of_chain, policy, priority, rules, what};
use super::row::Row;
use crate::ui::helpers::layout::column;

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec![
            "KIND", "WHAT", "TYPE", "HOOK", "PRIORITY", "POLICY", "CHAINS", "RULES",
        ]),
        false => TableRow::new(vec!["KIND", "WHAT", "HOOK", "PRIORITY", "POLICY", "RULES"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(5),
        ],
        false => vec![
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(5),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool, columns: &[usize]) -> TableRow<'static> {
    let mut cells = vec![row.kind.name().to_string(), what(row)];
    if wide {
        cells.push(kind_of_chain(row));
    }
    cells.push(hook(row));
    cells.push(priority(row));
    cells.push(policy(row));
    if wide {
        cells.push(chains(row));
    }
    cells.push(rules(row));

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
