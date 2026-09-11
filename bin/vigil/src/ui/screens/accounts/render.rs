use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::cells::cells;
use super::columns::{header, widths};
use super::notices::{empty, missing};
use super::regions::{middle, split_about, split_bottom, split_menu, split_top};
use super::rows::rows;
use super::showing::Showing;
use super::tally::tally;
use crate::ui::helpers::layout::listing;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Arrows, Look, Reading, Search, Subject, View};

const ROOM_FOR_THE_SOURCE: u16 = 100;

const LINES_OF_DEFINITION: usize = 2;

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    let (menu, rest) = split_menu(area);
    if let Some(menu) = menu {
        Paragraph::new(row_of_names(look, view, showing)).render(menu, buffer);
    }

    if let Some(notice) = missing(view) {
        notice.render(look, rest, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading("users") else {
        return;
    };

    let said: Vec<Line<'static>> = wrap::wrap(
        showing.subject.about(),
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

    let (box_, rest) = split_top(rest, showing.search);
    let rows = rows(view, showing.subject, showing.search);
    let notice = rows.is_empty().then(|| empty(view, showing));
    let (table, footer) = split_bottom(rest, look, middle(&notice, look, &rows, rest.width));

    if let Some(box_) = box_ {
        Paragraph::new(showing.search.line(look)).render(box_, buffer);
    }

    match notice {
        Some(notice) => notice.render(look, table, buffer),
        None => {
            let wide = area.width >= ROOM_FOR_THE_SOURCE;
            let widths = widths(showing.subject, wide);
            let columns = listing::column_widths(look, &widths, table);
            listing::render(
                look,
                header(showing.subject, wide),
                rows.iter()
                    .map(|row| cells(showing.subject, row, view, wide, &columns))
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
        tally(view, showing, snapshot, rows.len(), footer.width),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
}

fn row_of_names(look: Look, view: &View, showing: &Showing<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match showing.arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (index, subject) in Subject::on(view).into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match subject == showing.subject {
            true => spans.push(Span::styled(
                format!("[{}]", subject.name()),
                match showing.arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {} ", subject.name()))),
        }
    }
    Line::from(spans)
}

pub fn printed_height(
    view: &View,
    look: Look,
    subject: Subject,
    search: &Search,
    width: u16,
) -> u16 {
    let showing = Showing {
        subject,
        search,
        cursor: 0,
        elsewhere: 0,
        arrows: Arrows::Away,
    };
    let rows = rows(view, subject, search);
    let about = wrap::wrap(subject.about(), (width as usize).saturating_sub(4))
        .len()
        .min(LINES_OF_DEFINITION) as u16;
    let notice = rows.is_empty().then(|| empty(view, &showing));
    let middle = middle(&notice, look, &rows, width).max(2) as u16;
    1 + about + middle + 1 + 1
}
