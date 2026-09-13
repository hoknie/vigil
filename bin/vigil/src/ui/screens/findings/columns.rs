use ratatui::layout::Constraint;
use ratatui::widgets::Row;
use vigil_model::Finding;

use super::shape::Shape;

pub(super) fn header(shape: Shape, picking: bool) -> Row<'static> {
    let mut named = match shape {
        Shape::Roomy => vec!["TIME", "SEVERITY", "KIND", "TITLE", "OBJECT", "SEEN"],
        Shape::Plain => vec!["TIME", "SEVERITY", "KIND", "TITLE"],
        Shape::Cramped => vec!["TIME", "SEVERITY", "TITLE"],
    };
    if picking {
        named.insert(0, "");
    }
    Row::new(named)
}

pub(super) fn widths(shape: Shape, severity: u16, picking: bool) -> Vec<Constraint> {
    let mut widths = shaped(shape, severity);
    if picking {
        widths.insert(0, Constraint::Length(1));
    }
    widths
}

fn shaped(shape: Shape, severity: u16) -> Vec<Constraint> {
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
