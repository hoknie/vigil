use ratatui::widgets::Row as TableRow;

use super::arrangement::Arrangement;
use super::fields::{basename, command, endpoint, program, protocol, user};
use super::row::Row;
use super::what::What;
use crate::ui::helpers::layout::column;

pub(super) fn cells(
    row: &Row<'_>,
    arrangement: Arrangement,
    wide: bool,
    columns: &[usize],
) -> TableRow<'static> {
    let mut cells: Vec<String> = match (&row.what, arrangement) {
        (What::Socket(item), Arrangement::Flat) => {
            let mut cells = vec![
                protocol(item, &row.key).to_string(),
                endpoint(item, &row.key),
                user(item),
                program(item),
            ];
            if wide {
                cells.push(command(item));
            }
            cells
        }
        (What::Socket(item), Arrangement::ByProgram) => {
            let mut cells = vec![
                format!("  {}", protocol(item, &row.key)),
                endpoint(item, &row.key),
                user(item),
            ];
            if wide {
                cells.push(String::new());
            }
            cells
        }
        (
            What::Program {
                path,
                count,
                ambiguous,
            },
            _,
        ) => {
            let mut cells = vec![
                match ambiguous {
                    true => format!("{path} ({count})"),
                    false => format!("{} ({count})", basename(path)),
                },
                String::new(),
                String::new(),
            ];
            if wide {
                cells.push(path.clone());
            }
            cells
        }
        (What::Unresolved { count }, _) => {
            let mut cells = vec![
                format!("owner not resolved ({count})"),
                "permission denied".to_string(),
                String::new(),
            ];
            if wide {
                cells.push(String::new());
            }
            cells
        }
    };

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
