use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{cells, header, widths};
use super::notices::{about, empty, gone, missing};
use super::rows::{COLLECTOR, rows};
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::{listing, panes, wrap};
use crate::ui::{Arrows, Look, Reading, View};

const ROOM_FOR_THE_TYPE: u16 = 118;

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
            let notice = gone(missing);
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

    let said = said(view, look, area.width);
    let (about, rest) = panes::about(rest, look, said.len() as u16);
    if let Some(about) = about {
        Paragraph::new(said).render(about, buffer);
    }

    let (box_, rest) = panes::search_box(rest, showing.search);
    let rows = rows(view, showing);
    let (table, footer) = panes::footer(rest, look, rows.len() + 1);

    if let Some(box_) = box_ {
        Paragraph::new(showing.search.line(look)).render(box_, buffer);
    }

    match rows.is_empty() {
        true => empty(view, showing).render(look, table, buffer),
        false => {
            let wide = area.width >= ROOM_FOR_THE_TYPE;
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

fn said(view: &View, look: Look, width: u16) -> Vec<Line<'static>> {
    about(view)
        .into_iter()
        .flat_map(|sentence| wrap::wrap(&sentence, (width as usize).saturating_sub(4)))
        .map(|part| Line::styled(format!("   {part}"), look.palette.quiet()))
        .collect()
}
