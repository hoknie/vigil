use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use super::group::group;
use super::key::key;
use super::lines::silencing;
use super::session::session;
use super::session_source::session_source;
use super::sudoer::sudoer;
use super::unknown::unknown;
use super::user::account;
use crate::ui::screens::accounts::{Kind, Row};
use crate::ui::{Look, Notice, Report, View};

pub fn render(
    row: Option<&Row<'_>>,
    view: &View,
    look: Look,
    top: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    let Some(row) = row else {
        Notice::plain("Nothing is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(row, view, look, look.text_width(area.width));
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

pub fn height(row: Option<&Row<'_>>, view: &View, look: Look, width: usize) -> usize {
    match row {
        Some(row) => report(row, view, look, width).len(),
        None => 0,
    }
}

fn report(row: &Row<'_>, view: &View, look: Look, width: usize) -> Report {
    let mut report = Report::default();
    match row.kind {
        Kind::Account => account(&mut report, row, view, look, width),
        Kind::Group => group(&mut report, row, view, look, width),
        Kind::Sudoer => sudoer(&mut report, row, view, look, width),
        Kind::Key => key(&mut report, row, look, width),
        Kind::Session => session(&mut report, row, look, width),
        Kind::SessionSource => session_source(&mut report, row, look, width),
        Kind::Unknown => unknown(&mut report, row, look, width),
    }
    silencing(&mut report, &row.key, look, width);
    report
}
