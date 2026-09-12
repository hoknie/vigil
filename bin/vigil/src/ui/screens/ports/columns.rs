use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::arrangement::Arrangement;

pub(super) fn header(arrangement: Arrangement, wide: bool) -> TableRow<'static> {
    match (arrangement, wide) {
        (Arrangement::Flat, true) => {
            TableRow::new(vec!["PROTO", "ADDRESS", "USER", "PROGRAM", "COMMAND"])
        }
        (Arrangement::Flat, false) => TableRow::new(vec!["PROTO", "ADDRESS", "USER", "PROGRAM"]),
        (Arrangement::ByProgram, true) => {
            TableRow::new(vec!["PROGRAM", "ADDRESS", "USER", "WHERE IT IS"])
        }
        (Arrangement::ByProgram, false) => TableRow::new(vec!["PROGRAM", "ADDRESS", "USER"]),
    }
}

pub(super) fn widths(arrangement: Arrangement, wide: bool) -> Vec<Constraint> {
    match (arrangement, wide) {
        (Arrangement::Flat, true) => vec![
            Constraint::Length(5),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ],
        (Arrangement::Flat, false) => vec![
            Constraint::Length(5),
            Constraint::Min(22),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
        (Arrangement::ByProgram, true) => vec![
            Constraint::Min(22),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
        (Arrangement::ByProgram, false) => vec![
            Constraint::Min(20),
            Constraint::Fill(1),
            Constraint::Length(10),
        ],
    }
}
