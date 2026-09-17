use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Row as TableRow, Widget};
use vigil_view::{Pane, Room, Section};

use super::chooser;
use super::columns::{constraints, header};
use super::menu::row_of_names;
use super::notices::{gone, missing, nothing_here, said};
use super::regions::{split_about, split_bottom, split_menu, split_top};
use super::showing::{Showing, asked};
use crate::ui::helpers::layout::{column, footing, listing, panes, wrap};
use crate::ui::types::cache::Shown;
use crate::ui::{Arrows, Look, Placed, Reading, View};

const LINES_OF_DEFINITION: usize = 2;

const MARKED: &str = "x";

pub fn render(
    view: &View,
    look: Look,
    section: &dyn Section,
    showing: &Showing<'_>,
    area: Rect,
    buffer: &mut Buffer,
) -> Placed {
    let mut placed = Placed::default();
    let panes = section.panes();
    if panes.is_empty() {
        crate::ui::Notice::plain("Nothing is listed here.")
            .saying("The agent is not reporting a reading for this section.")
            .render(look, area, buffer);
        return placed;
    }
    let Some(pane) = panes.get(showing.at.min(panes.len() - 1)) else {
        return placed;
    };

    let shown: Vec<usize> = match view.reading(pane.reads()) {
        Reading::Taken(reading) => (0..panes.len())
            .filter(|index| panes[*index].shown(reading))
            .collect(),
        _ => (0..panes.len()).collect(),
    };

    let area = match showing.gone {
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

    let (menu, rest) = split_menu(area);
    if let Some(menu) = menu
        && shown.len() > 1
    {
        let (names, places) = row_of_names(look, &panes, &shown, showing.at, showing.arrows);
        Paragraph::new(names).render(menu, buffer);
        placed.names = places
            .into_iter()
            .map(|(at, x, wide)| {
                (
                    at,
                    Rect::new(menu.x + x, menu.y, wide, 1).intersection(menu),
                )
            })
            .collect();
    }

    let arrangements = pane.arrangements();

    if let Some(notice) = missing(view, pane.as_ref()) {
        notice.render(look, rest, buffer);
        return placed;
    }
    let Reading::Taken(snapshot) = view.reading(pane.reads()) else {
        return placed;
    };

    let about_said = match arrangements
        .iter()
        .find(|one| Some(one.name) == showing.arranged)
        .and_then(|one| one.about)
    {
        Some(more) => format!("{} · {more}", pane.about()),
        None => pane.about().to_string(),
    };
    let lines_of_definition = match about_said.len() > pane.about().len() {
        true => LINES_OF_DEFINITION + 1,
        false => LINES_OF_DEFINITION,
    };

    let said_about: Vec<Line<'static>> =
        wrap::wrap(&about_said, (area.width as usize).saturating_sub(4))
            .into_iter()
            .take(lines_of_definition)
            .map(|part| Line::styled(format!("   {part}"), look.palette.quiet()))
            .collect();

    let (about, rest) = split_about(rest, look, said_about.len() as u16);
    if let Some(about) = about {
        Paragraph::new(said_about).render(about, buffer);
    }

    let toggles = pane.toggles();
    let (chooser_area, searching, rest) = split_top(rest, showing.search);
    let hidden = showing.hidden();
    let opened = showing.opened();
    let asked = asked(showing, &hidden, &opened);
    let shown = match &showing.listed {
        Some(listed) => Rc::clone(listed),
        None => Rc::new(Shown::built(pane.rows(snapshot, &asked))),
    };
    let rows = shown.rows();
    let marking = pane.offers().marking && look.interactive();
    let (table, footer) = split_bottom(rest, look, rows.len());

    if let Some(area) = chooser_area
        && !(toggles.is_empty() && arrangements.is_empty())
    {
        Paragraph::new(chooser::line(
            look,
            &arrangements,
            showing.arranged,
            &toggles,
            showing.hidden,
        ))
        .render(area, buffer);
    }
    if let Some(area) = searching {
        Paragraph::new(showing.search.line(look)).render(area, buffer);
    }

    let room = Room::of(area.width);
    if rows.is_empty() {
        match snapshot.items.is_empty() {
            true => nothing_here(pane.as_ref(), &snapshot.taken_at).render(look, table, buffer),
            false => said(pane.empty(&asked)).render(look, table, buffer),
        }
    } else {
        let widths = constraints(&pane.columns(room), marking);
        let fitted = listing::column_widths(look, &widths, table);
        let draw = |at: usize| {
            drawn(
                pane.as_ref(),
                snapshot,
                &rows.get(at).expect("a row the list counted"),
                room,
                &fitted,
                marking,
                showing,
            )
        };
        placed.rows = listing::render(
            look,
            header(&pane.columns(room), marking),
            listing::Rows {
                total: rows.len(),
                drawn: &draw,
            },
            &widths,
            listing::Where {
                at: showing.cursor,
                focused: showing.arrows == Arrows::List,
            },
            table,
            buffer,
        );
    }

    footing::render(
        look,
        trimmed(
            match &showing.tally {
                Some(tally) => tally.clone(),
                None => pane.tally(snapshot, &asked, rows.len()),
            },
            footer.width,
        ),
        footer,
        buffer,
    );
    placed
}

pub fn printed_height(
    view: &View,
    look: Look,
    section: &dyn Section,
    showing: &Showing<'_>,
    width: u16,
) -> u16 {
    let panes = section.panes();
    let Some(pane) = panes.get(showing.at.min(panes.len().saturating_sub(1))) else {
        return 0;
    };
    let hidden = showing.hidden();
    let opened = showing.opened();
    let asked = asked(showing, &hidden, &opened);
    let rows = match view.reading(pane.reads()) {
        Reading::Taken(snapshot) => pane.rows(snapshot, &asked).len(),
        _ => 0,
    };
    let about = wrap::wrap(pane.about(), (width as usize).saturating_sub(4))
        .len()
        .min(LINES_OF_DEFINITION) as u16;
    let table = (rows.max(2) as u16).saturating_add(1);
    let _ = look;

    1 + about + table + 1 + 1
}

fn drawn(
    pane: &dyn Pane,
    snapshot: &vigil_model::Snapshot,
    row: &vigil_view::RowKey,
    room: Room,
    fitted: &[usize],
    marking: bool,
    showing: &Showing<'_>,
) -> TableRow<'static> {
    let ahead = usize::from(marking);
    let mut cells: Vec<String> = Vec::with_capacity(ahead + 1);
    if marking {
        cells.push(match showing.is_marked(&row.key) && row.of_the_reading {
            true => MARKED.to_string(),
            false => " ".to_string(),
        });
    }
    cells.extend(
        pane.cells(snapshot, row, room)
            .into_iter()
            .enumerate()
            .map(|(index, cell)| match fitted.get(index + ahead) {
                Some(width) => column::fit(&cell.text, *width),
                None => cell.text,
            }),
    );

    TableRow::new(cells)
}

fn trimmed(tally: String, width: u16) -> String {
    let room = (width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in tally.split(" · ") {
        let next = match line.is_empty() {
            true => part.to_string(),
            false => format!("{line} · {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    format!(" {line}")
}
