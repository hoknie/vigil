use ratatui::layout::Constraint;
use ratatui::widgets::Row;

const ROOM_FOR_THE_FILE: u16 = 118;

const ROOM_FOR_THE_KIND: u16 = 72;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    Cramped,
    Plain,
    Roomy,
}

impl Shape {
    pub(super) fn of(width: u16) -> Shape {
        match width {
            width if width >= ROOM_FOR_THE_FILE => Shape::Roomy,
            width if width >= ROOM_FOR_THE_KIND => Shape::Plain,
            _ => Shape::Cramped,
        }
    }
}

pub(super) fn header(shape: Shape) -> Row<'static> {
    Row::new(match shape {
        Shape::Roomy => vec!["STANDING", "OBJECT", "KIND", "UNTIL", "REASON", "FILE"],
        Shape::Plain => vec!["STANDING", "OBJECT", "KIND", "REASON"],
        Shape::Cramped => vec!["STANDING", "OBJECT", "REASON"],
    })
}

pub(super) fn widths(shape: Shape) -> Vec<Constraint> {
    match shape {
        Shape::Roomy => vec![
            Constraint::Length(15),
            Constraint::Fill(3),
            Constraint::Fill(2),
            Constraint::Length(16),
            Constraint::Fill(3),
            Constraint::Fill(2),
        ],
        Shape::Plain => vec![
            Constraint::Length(15),
            Constraint::Fill(3),
            Constraint::Fill(2),
            Constraint::Fill(3),
        ],
        Shape::Cramped => vec![
            Constraint::Length(15),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ],
    }
}
