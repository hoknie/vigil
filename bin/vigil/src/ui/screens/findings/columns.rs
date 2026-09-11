use ratatui::layout::Constraint;
use ratatui::widgets::Row;
use vigil_model::Finding;

use super::shape::Shape;

pub(super) fn header(shape: Shape) -> Row<'static> {
    match shape {
        Shape::Roomy => Row::new(vec!["TIME", "SEVERITY", "KIND", "TITLE", "OBJECT", "SEEN"]),
        Shape::Plain => Row::new(vec!["TIME", "SEVERITY", "KIND", "TITLE"]),
        Shape::Cramped => Row::new(vec!["TIME", "SEVERITY", "TITLE"]),
    }
}

pub(super) fn widths(shape: Shape, severity: u16) -> Vec<Constraint> {
    match shape {
        Shape::Roomy => vec![
            Constraint::Length(8),
            Constraint::Length(severity),
            Constraint::Min(24),
            Constraint::Fill(3),
            Constraint::Fill(2),
            Constraint::Length(4),
        ],
        Shape::Plain => vec![
            Constraint::Length(8),
            Constraint::Length(severity),
            Constraint::Min(22),
            Constraint::Fill(1),
        ],
        Shape::Cramped => vec![
            Constraint::Length(8),
            Constraint::Length(severity),
            Constraint::Fill(1),
        ],
    }
}

pub(super) fn severity_width(passing: &[&Finding]) -> u16 {
    passing
        .iter()
        .map(|finding| finding.severity.as_str().chars().count() as u16)
        .max()
        .unwrap_or(8)
        .clamp(8, 20)
}
