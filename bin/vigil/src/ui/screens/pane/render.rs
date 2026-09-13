use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Row as TableRow, Widget};
use vigil_view::{Column, Pane, Room, Section, Width};

use super::chooser;
use super::menu::row_of_names;
use super::notices::{gone, missing, nothing_here, said};
use super::regions::{split_about, split_bottom, split_menu, split_top};
use super::showing::{Showing, asked};
use crate::ui::helpers::layout::{column, listing, panes, wrap};
use crate::ui::{Arrows, Look, Reading, View};

const LINES_OF_DEFINITION: usize = 2;

pub fn render(
    view: &View,
    look: Look,
    section: &dyn Section,
    showing: &Showing<'_>,
    area: Rect,
    buffer: &mut Buffer,
) {
    let panes = section.panes();
    let Some(pane) = panes.get(showing.at.min(panes.len().saturating_sub(1))) else {
        return;
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
        Paragraph::new(row_of_names(
            look,
            &panes,
            &shown,
            showing.at,
            showing.arrows,
        ))
        .render(menu, buffer);
    }

    let arrangements = pane.arrangements();

    if let Some(notice) = missing(view, pane.as_ref()) {
        notice.render(look, rest, buffer);
        return;
    }
    let Reading::Taken(snapshot) = view.reading(pane.reads()) else {
        return;
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
    let asked = asked(showing, &hidden);
    let rows = pane.rows(snapshot, &asked);
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
        let widths = constraints(&pane.columns(room));
        let fitted = listing::column_widths(look, &widths, table);
        listing::render(
            look,
            header(&pane.columns(room)),
            rows.iter()
                .map(|row| drawn(pane.as_ref(), snapshot, row, room, &fitted))
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
        footing(pane.tally(snapshot, &asked, rows.len()), footer.width),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
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
    let asked = asked(showing, &hidden);
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
) -> TableRow<'static> {
    let cells: Vec<String> = pane
        .cells(snapshot, row, room)
        .into_iter()
        .enumerate()
        .map(|(index, cell)| match fitted.get(index) {
            Some(width) => column::fit(&cell.text, *width),
            None => cell.text,
        })
        .collect();

    TableRow::new(cells)
}

fn header(columns: &[Column]) -> TableRow<'static> {
    TableRow::new(
        columns
            .iter()
            .map(|column| column.header)
            .collect::<Vec<&'static str>>(),
    )
}

fn constraints(columns: &[Column]) -> Vec<Constraint> {
    columns
        .iter()
        .map(|column| match column.width {
            Width::Fixed(room) => Constraint::Length(room),
            Width::Least(room) => Constraint::Min(room),
            Width::Share(part) => Constraint::Fill(part),
        })
        .collect()
}

fn footing(tally: String, width: u16) -> String {
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
