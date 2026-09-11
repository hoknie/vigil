use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::notices::missing;
use super::rows::rows;
use super::showing::Showing;
use super::tally::tally;
use super::{launches, running};
use crate::ui::helpers::layout::listing;
use crate::ui::helpers::layout::panes;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Arrows, Look, Notice, Program, Reading, Search, View};

const ROOM_FOR_THE_PATH: u16 = 118;

const LINES_OF_DEFINITION: usize = 2;

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    let (menu, rest) = panes::menu(area);
    if let Some(menu) = menu {
        Paragraph::new(row_of_names(look, showing)).render(menu, buffer);
    }

    if let Some(notice) = missing(view, showing) {
        notice.render(look, rest, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading(showing.program.collector()) else {
        return;
    };

    let said: Vec<Line<'static>> = wrap::wrap(
        showing.program.about(),
        (area.width as usize).saturating_sub(4),
    )
    .into_iter()
    .take(LINES_OF_DEFINITION)
    .map(|part| Line::styled(format!("   {part}"), look.palette.quiet()))
    .collect();

    let (about, rest) = panes::about(rest, look, said.len() as u16);
    if let Some(about) = about {
        Paragraph::new(said).render(about, buffer);
    }

    let (box_, rest) = panes::search_box(rest, showing.search);
    let rows = rows(view, showing.program, showing.search);
    let (table, footer) = panes::footer(rest, look, rows.len() + 1);

    if let Some(box_) = box_ {
        Paragraph::new(showing.search.line(look)).render(box_, buffer);
    }

    match rows.is_empty() {
        true => empty(view, showing).render(look, table, buffer),
        false => {
            let wide = area.width >= ROOM_FOR_THE_PATH;
            let widths = match showing.program {
                Program::Running => running::widths(wide),
                Program::Launches => launches::widths(wide),
            };
            let columns = listing::column_widths(look, &widths, table);
            listing::render(
                look,
                match showing.program {
                    Program::Running => running::header(wide),
                    Program::Launches => launches::header(wide),
                },
                rows.iter()
                    .map(|row| match showing.program {
                        Program::Running => running::cells(row, wide, &columns),
                        Program::Launches => launches::cells(row, wide, &columns),
                    })
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

fn empty(view: &View, showing: &Showing<'_>) -> Notice {
    match showing.program {
        Program::Running => running::empty(view, showing.search),
        Program::Launches => launches::empty(view, showing.search),
    }
}

pub fn printed_height(view: &View, program: Program, search: &Search, width: u16) -> u16 {
    let rows = rows(view, program, search);
    let about = wrap::wrap(program.about(), (width as usize).saturating_sub(4))
        .len()
        .min(LINES_OF_DEFINITION) as u16;
    1 + about + (rows.len() as u16 + 1).max(2) + 1 + 1
}

fn row_of_names(look: Look, showing: &Showing<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match showing.arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (index, program) in Program::ALL.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match *program == showing.program {
            true => spans.push(Span::styled(
                format!("[{}]", program.name()),
                match showing.arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {} ", program.name()))),
        }
    }
    Line::from(spans)
}
