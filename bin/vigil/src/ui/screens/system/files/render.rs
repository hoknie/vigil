use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::super::showing::Showing;
use super::columns::{cells, header, widths};
use super::notices::{empty, missing};
use super::rows::{COLLECTOR, rows};
use super::tally::tally;
use crate::ui::helpers::layout::{listing, panes};
use crate::ui::helpers::words::gone;
use crate::ui::{Arrows, Look, Reading, View};

const ROOM_FOR_THE_HASH: u16 = 118;

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    if let Some(notice) = missing(view) {
        notice.render(look, area, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return;
    };

    let rest = match showing.gone {
        None => area,
        Some(missing) => {
            let notice = gone::out_of_the_reading(missing);
            let lines = notice.lines(look, area.width as usize);
            let (band, rest) = panes::about(area, look, lines.len() as u16 + 1);
            match band {
                Some(band) => {
                    Paragraph::new(lines).render(band, buffer);
                    rest
                }
                None => area,
            }
        }
    };

    let (box_, rest) = panes::search_box(rest, showing.search);
    let rows = rows(view, showing);
    let (table, footer) = panes::footer(rest, look, rows.len() + 1);

    if let Some(box_) = box_ {
        Paragraph::new(showing.search.line(look)).render(box_, buffer);
    }

    match rows.is_empty() {
        true => empty(showing).render(look, table, buffer),
        false => {
            let wide = area.width >= ROOM_FOR_THE_HASH;
            let widths = widths(wide);
            let columns = listing::column_widths(look, &widths, table);
            listing::render(
                look,
                header(wide),
                rows.iter().map(|row| cells(row, wide, &columns)).collect(),
                &widths,
                listing::Where {
                    at: showing.cursor,
                    focused: showing.arrows == Arrows::List,
                },
                table,
                buffer,
            );
        }
    }

    Paragraph::new(Line::styled(
        tally(view, showing, snapshot, footer.width),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
}
