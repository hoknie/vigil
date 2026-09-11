use ratatui::text::Span;
use ratatui::widgets::{Cell, Row};
use vigil_model::Finding;

use super::shape::Shape;
use crate::ui::Look;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::words::moment;

pub fn keys(passing: &[&Finding]) -> Vec<String> {
    passing
        .iter()
        .map(|finding| finding.event_id.clone())
        .collect()
}

pub(super) fn row(finding: &Finding, look: Look, shape: Shape, columns: &[usize]) -> Row<'static> {
    let cut = |index: usize, text: String| match columns.get(index) {
        Some(width) => column::fit(&text, *width),
        None => text,
    };

    let mut cells = vec![
        Cell::from(cut(
            0,
            moment::time_of_day(&finding.observed_at).to_string(),
        )),
        Cell::from(Span::styled(
            cut(1, finding.severity.as_str().to_string()),
            look.palette.severity(&finding.severity),
        )),
    ];

    let mut at = 2;
    if shape != Shape::Cramped {
        cells.push(Cell::from(cut(at, finding.kind.as_str().to_string())));
        at += 1;
    }
    cells.push(Cell::from(cut(at, finding.title.clone())));
    if shape == Shape::Roomy {
        cells.push(Cell::from(cut(at + 1, finding.finding_key.clone())));
        cells.push(Cell::from(cut(at + 2, finding.occurrences.to_string())));
    }
    Row::new(cells)
}
