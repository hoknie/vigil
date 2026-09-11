use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use super::collectors::collectors;
use super::identity::identity;
use super::limitations::limitations;
use super::reporters::reporters;
use super::silence::silence;
use super::storage::storage;
use crate::ui::{Look, Report, View};

pub fn render(view: &View, look: Look, top: usize, area: Rect, buffer: &mut Buffer) {
    if view.status.is_none() {
        return;
    }

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(view, look, look.text_width(area.width));
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

pub fn height(view: &View, look: Look, width: usize) -> usize {
    match view.status.is_some() {
        true => report(view, look, width).len(),
        false => 0,
    }
}

fn report(view: &View, look: Look, width: usize) -> Report {
    let mut report = Report::default();
    let Some(status) = &view.status else {
        return report;
    };

    identity(&mut report, view, look, width);
    collectors(&mut report, &status.agent, look, width);
    reporters(&mut report, &status.agent, look, width);
    storage(&mut report, &status.agent, look, width);
    silence(&mut report, &status.agent, look, width);
    limitations(&mut report, &status.agent, look, width);

    report
}
