use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use super::listener::socket;
use super::program::program;
use super::unresolved::unresolved;
use crate::ui::screens::ports::{Row, What};
use crate::ui::{Look, Notice, Report};

pub fn render(row: Option<&Row<'_>>, look: Look, top: usize, area: Rect, buffer: &mut Buffer) {
    let Some(row) = row else {
        Notice::plain("No socket is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(row, look, look.text_width(area.width));
    let page = area.height as usize;
    let top = top.min(report.len().saturating_sub(page));

    Paragraph::new(
        report
            .lines()
            .iter()
            .skip(top)
            .take(page)
            .cloned()
            .collect::<Vec<Line>>(),
    )
    .render(
        Rect {
            width: area.width - gutter,
            ..area
        },
        buffer,
    );

    if gutter > 0 && report.len() > page {
        let mut bar = ScrollbarState::new(report.len() - page).position(top);
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(look.palette.border())
                .begin_symbol(None)
                .end_symbol(None),
            area,
            buffer,
            &mut bar,
        );
    }
}

pub fn height(row: Option<&Row<'_>>, look: Look, width: usize) -> usize {
    match row {
        Some(row) => report(row, look, width).len(),
        None => 0,
    }
}

fn report(row: &Row<'_>, look: Look, width: usize) -> Report {
    let mut report = Report::default();
    match &row.what {
        What::Socket(item) => socket(&mut report, &row.key, item, look, width),
        What::Program { path, count, .. } => program(&mut report, path, *count, look, width),
        What::Unresolved { count } => unresolved(&mut report, *count, look, width),
    }
    report
}
