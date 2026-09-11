use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{header, severity_width, widths};
use super::notices::{notice, searching};
use super::regions::{split_bottom, split_top};
use super::rows::row;
use super::shape::Shape;
use super::tally::tally;
use crate::ui::helpers::layout::listing;
use crate::ui::{Filter, Look, View};

pub fn render(
    view: &View,
    filter: &Filter,
    look: Look,
    cursor: usize,
    focused: bool,
    area: Rect,
    buffer: &mut Buffer,
) {
    if !view.has_reading() {
        return;
    }

    let passing = filter.passing(&view.found.findings);
    let (search, rest) = split_top(area, filter);
    let (table, footer) = split_bottom(rest, look, passing.len());

    if let Some(search) = search {
        Paragraph::new(searching(filter, look)).render(search, buffer);
    }

    if passing.is_empty() {
        notice(view, filter).render(look, table, buffer);
    } else {
        let shape = Shape::of(area.width);
        let widths = widths(shape, severity_width(&passing));
        let columns = listing::column_widths(look, &widths, table);
        listing::render(
            look,
            header(shape),
            passing
                .iter()
                .map(|finding| row(finding, look, shape, &columns))
                .collect(),
            &widths,
            listing::Where {
                at: cursor,
                focused,
            },
            table,
            buffer,
        );
    }

    Paragraph::new(Line::styled(
        tally(view, filter, footer),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
}
