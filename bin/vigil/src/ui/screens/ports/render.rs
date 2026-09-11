use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::arrangement::Arrangement;
use super::cells::cells;
use super::columns::{header, widths};
use super::notices::{empty, missing};
use super::regions::{split_about, split_bottom, split_menu, split_top};
use super::rows::rows;
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::listing;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Arrows, Look, Reading, View};

const ROOM_FOR_THE_COMMAND: u16 = 118;

const LINES_OF_DEFINITION: usize = 2;

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    let (menu, rest) = split_menu(area);
    if let Some(menu) = menu {
        Paragraph::new(row_of_names(look, showing)).render(menu, buffer);
    }

    if let Some(notice) = missing(view) {
        notice.render(look, rest, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading("ports") else {
        return;
    };

    let said: Vec<Line<'static>> = wrap::wrap(
        showing.arrangement.about(),
        (area.width as usize).saturating_sub(4),
    )
    .into_iter()
    .take(LINES_OF_DEFINITION)
    .map(|part| Line::styled(format!("   {part}"), look.palette.quiet()))
    .collect();

    let (about, rest) = split_about(rest, look, said.len() as u16);
    if let Some(about) = about {
        Paragraph::new(said).render(about, buffer);
    }

    let (chooser, searching, rest) = split_top(rest, showing);
    let rows = rows(view, showing);
    let (table, footer) = split_bottom(rest, look, rows.len());

    if let Some(chooser) = chooser {
        Paragraph::new(showing.protocols.line(look)).render(chooser, buffer);
    }
    if let Some(searching) = searching {
        Paragraph::new(showing.search.line(look)).render(searching, buffer);
    }

    if rows.is_empty() {
        empty(showing).render(look, table, buffer);
    } else {
        let wide = area.width >= ROOM_FOR_THE_COMMAND;
        let widths = widths(showing.arrangement, wide);
        let columns = listing::column_widths(look, &widths, table);
        listing::render(
            look,
            header(showing.arrangement, wide),
            rows.iter()
                .map(|row| cells(row, showing.arrangement, wide, &columns))
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

    Paragraph::new(Line::styled(
        tally(snapshot, showing, rows.len(), footer.width),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
}

fn row_of_names(look: Look, showing: &Showing<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match showing.arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (index, arrangement) in Arrangement::ALL.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match *arrangement == showing.arrangement {
            true => spans.push(Span::styled(
                format!("[{}]", arrangement.name()),
                match showing.arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {} ", arrangement.name()))),
        }
    }
    Line::from(spans)
}
