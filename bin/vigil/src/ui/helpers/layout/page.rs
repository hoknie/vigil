use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use crate::ui::{Look, Report};

pub fn render(report: &Report, look: Look, top: usize, area: Rect, buffer: &mut Buffer) {
    let gutter = area.width - look.text_width(area.width) as u16;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn long() -> Report {
        let mut report = Report::default();
        for number in 0..40 {
            report.push(Line::raw(format!("line {number}")));
        }
        report
    }

    #[test]
    fn a_reader_scrolled_past_the_end_is_put_on_the_last_page_rather_than_on_nothing() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 10));

        render(&long(), fixture::look(), 900, buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        assert!(page.contains("line 39"), "{page}");
    }
}
