use ratatui::text::Span;
use ratatui::widgets::{Cell, Row};
use vigil_model::Finding;
use vigil_view::time_of_day;

use super::shape::Shape;
use crate::ui::Look;
use crate::ui::helpers::layout::column;

const PICKED: &str = "\u{d7}";

pub fn keys(passing: &[&Finding]) -> Vec<String> {
    passing
        .iter()
        .map(|finding| finding.event_id.clone())
        .collect()
}

pub(super) fn row(
    finding: &Finding,
    look: Look,
    shape: Shape,
    columns: &[usize],
    picked: Option<bool>,
) -> Row<'static> {
    let cut = |index: usize, text: String| match columns.get(index) {
        Some(width) => column::fit(&text, *width),
        None => text,
    };

    let mut cells = Vec::new();
    let mut at = 0;

    if let Some(picked) = picked {
        cells.push(Cell::from(match picked {
            true => PICKED,
            false => " ",
        }));
        at += 1;
    }

    cells.push(Cell::from(cut(
        at,
        time_of_day(&finding.observed_at).to_string(),
    )));
    cells.push(Cell::from(Span::styled(
        cut(at + 1, finding.severity.as_str().to_string()),
        look.palette.severity(&finding.severity),
    )));
    at += 2;

    if shape != Shape::Cramped {
        cells.push(Cell::from(cut(at, finding.kind.as_str().to_string())));
        at += 1;
    }
    cells.push(Cell::from(cut(at, finding.title.clone())));
    if shape == Shape::Roomy {
        cells.push(Cell::from(cut(at + 1, finding.finding_key.clone())));
        cells.push(Cell::from(cut(at + 2, finding.occurrences.to_string())));
    }

    let row = Row::new(cells);
    match picked {
        Some(true) => row.style(look.palette.marked()),
        _ => row,
    }
}
