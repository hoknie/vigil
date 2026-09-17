use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{header, severity_width, widths};
use super::deeds::bar;
use super::notices::{notice, searching};
use super::regions::{split_bottom, split_top};
use super::rows::row;
use super::shape::Shape;
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::{footing, listing};
use crate::ui::{Look, View};

pub fn render(
    view: &View,
    look: Look,
    showing: &Showing<'_>,
    area: Rect,
    buffer: &mut Buffer,
) -> Vec<(usize, Rect)> {
    if !view.has_reading() {
        return Vec::new();
    }

    let Showing {
        filter,
        cursor,
        focused,
        sorting,
        picked,
        dismissed,
    } = *showing;
    let reported = filter.passing(&view.found.findings);
    let hidden = reported.len();
    let mut passing = dismissed.keeping(reported);
    let hidden = hidden - passing.len();
    super::sorting::sort(&mut passing, sorting);
    let picking = look.interactive() && !picked.is_empty();
    let (search, rest) = split_top(area, filter);
    let (table, footer, deeds) = split_bottom(rest, look, passing.len(), picking);

    if let Some(search) = search {
        Paragraph::new(searching(filter, look)).render(search, buffer);
    }

    let mut rows = Vec::new();
    if passing.is_empty() {
        notice(view, filter, hidden).render(look, table, buffer);
    } else {
        let shape = Shape::of(area.width);
        let widths = widths(shape, severity_width(&passing), picking);
        let columns = listing::column_widths(look, &widths, table);
        let draw = |at: usize| {
            let finding = passing[at];
            row(
                finding,
                look,
                shape,
                &columns,
                picking.then(|| picked.holds(&finding.event_id)),
            )
        };
        rows = listing::render(
            look,
            header(shape, picking),
            listing::Rows {
                total: passing.len(),
                drawn: &draw,
            },
            &widths,
            listing::Where {
                at: cursor,
                focused,
            },
            table,
            buffer,
        );
    }

    footing::render(
        look,
        tally(view, filter, sorting, passing.len(), hidden, footer),
        footer,
        buffer,
    );

    if let Some(deeds) = deeds {
        Paragraph::new(bar(picked, look, deeds.width)).render(deeds, buffer);
    }
    rows
}
