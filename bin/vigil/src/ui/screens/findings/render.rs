use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{header, severity_width, widths};
use super::deeds::bar;
use super::notices::{notice, searching};
use super::regions::{split_bottom, split_top};
use super::rows::row;
use super::shape::Shape;
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::listing;
use crate::ui::{Look, View};

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    if !view.has_reading() {
        return;
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

    if passing.is_empty() {
        notice(view, filter, hidden).render(look, table, buffer);
    } else {
        let shape = Shape::of(area.width);
        let widths = widths(shape, severity_width(&passing), picking);
        let columns = listing::column_widths(look, &widths, table);
        listing::render(
            look,
            header(shape, picking),
            passing
                .iter()
                .map(|finding| {
                    row(
                        finding,
                        look,
                        shape,
                        &columns,
                        picking.then(|| picked.holds(&finding.event_id)),
                    )
                })
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
        tally(view, filter, sorting, passing.len(), hidden, footer),
        look.palette.quiet(),
    ))
    .render(footer, buffer);

    if let Some(deeds) = deeds {
        Paragraph::new(bar(picked, look, deeds.width)).render(deeds, buffer);
    }
}
