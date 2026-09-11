use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{cells, header, widths};
use super::notices::{empty, missing};
use super::rows::{COLLECTOR, rows};
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::listing;
use crate::ui::helpers::layout::panes;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Arrows, Look, Reading, Startup, View};

const ROOM_FOR_THE_SOURCE: u16 = 118;

const LINES_OF_DEFINITION: usize = 2;

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    let (menu, rest) = panes::menu(area);
    if let Some(menu) = menu {
        Paragraph::new(row_of_names(look, view, showing)).render(menu, buffer);
    }

    if let Some(notice) = missing(view) {
        notice.render(look, rest, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading(COLLECTOR) else {
        return;
    };

    let said: Vec<Line<'static>> =
        wrap::wrap(definition(showing), (area.width as usize).saturating_sub(4))
            .into_iter()
            .take(LINES_OF_DEFINITION)
            .map(|part| Line::styled(format!("   {part}"), look.palette.quiet()))
            .collect();

    let (about, rest) = panes::about(rest, look, said.len() as u16);
    if let Some(about) = about {
        Paragraph::new(said).render(about, buffer);
    }

    let (chooser, rest) = match showing.list == Startup::Units {
        true => panes::menu(rest),
        false => (None, rest),
    };
    if let Some(chooser) = chooser {
        Paragraph::new(showing.nesting.line(look)).render(chooser, buffer);
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
            let wide = area.width >= ROOM_FOR_THE_SOURCE;
            let widths = widths(showing.list, wide);
            let columns = listing::column_widths(look, &widths, table);
            listing::render(
                look,
                header(showing.list, wide),
                rows.iter()
                    .map(|row| cells(showing.list, row, wide, &columns))
                    .collect(),
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

pub fn printed_height(view: &View, showing: &Showing<'_>, width: u16) -> u16 {
    let rows = rows(view, showing);
    let about = wrap::wrap(definition(showing), (width as usize).saturating_sub(4))
        .len()
        .min(LINES_OF_DEFINITION) as u16;
    let chooser = u16::from(showing.list == Startup::Units);
    1 + about + chooser + (rows.len() as u16 + 1).max(2) + 1 + 1
}

fn definition(showing: &Showing<'_>) -> &'static str {
    match showing.list == Startup::Units {
        true => showing
            .nesting
            .about()
            .unwrap_or_else(|| showing.list.about()),
        false => showing.list.about(),
    }
}

fn row_of_names(look: Look, view: &View, showing: &Showing<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match showing.arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (index, list) in Startup::on(view).into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match list == showing.list {
            true => spans.push(Span::styled(
                format!("[{}]", list.name()),
                match showing.arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {} ", list.name()))),
        }
    }
    Line::from(spans)
}
