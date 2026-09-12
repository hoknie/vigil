use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use super::report::report;
use super::subject::Subject;
use crate::ui::{Look, Notice};

pub fn render(
    subject: Option<Subject<'_>>,
    look: Look,
    top: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    let Some(subject) = subject else {
        Notice::plain("No row of this reading is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(&subject, look, look.text_width(area.width));
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

pub fn height(subject: Option<Subject<'_>>, look: Look, width: usize) -> usize {
    match subject {
        Some(subject) => report(&subject, look, width).len(),
        None => 0,
    }
}
