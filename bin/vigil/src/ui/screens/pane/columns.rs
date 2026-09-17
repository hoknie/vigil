use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;
use vigil_view::{Column, Width};

pub(super) fn header(columns: &[Column], marking: bool) -> TableRow<'static> {
    let mut headers: Vec<&'static str> = Vec::with_capacity(columns.len() + 1);
    if marking {
        headers.push(" ");
    }
    headers.extend(columns.iter().map(|column| column.header));
    TableRow::new(headers)
}

pub(super) fn constraints(columns: &[Column], marking: bool) -> Vec<Constraint> {
    let mut widths: Vec<Constraint> = Vec::with_capacity(columns.len() + 1);
    if marking {
        widths.push(Constraint::Length(1));
    }
    widths.extend(columns.iter().map(|column| match column.width {
        Width::Fixed(room) => Constraint::Length(room),
        Width::Least(room) => Constraint::Min(room),
        Width::Share(part) => Constraint::Fill(part),
    }));
    widths
}
