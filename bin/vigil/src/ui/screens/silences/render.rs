use std::path::Path;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row};

use super::columns::{Shape, header, widths};
use super::rows::{Shown, shown};
use super::standing::Standing;
use crate::ui::helpers::layout::{column, footing, listing};
use crate::ui::types::focus::silences::Silences;
use crate::ui::{Look, Notice};

pub const CAPTION: &str = "SILENCED ON THIS HOST";

pub fn render(
    silences: &Silences,
    running: Option<&[String]>,
    look: Look,
    area: Rect,
    buffer: &mut Buffer,
) -> Vec<(usize, Rect)> {
    if area.height < 2 {
        return Vec::new();
    }
    let table = Rect {
        height: area.height - 1,
        ..area
    };
    let footer = Rect {
        y: area.y + area.height - 1,
        height: 1,
        ..area
    };

    let rows = shown(silences, running);
    footing::render(
        look,
        tally(&rows, silences.at(), footer.width),
        footer,
        buffer,
    );

    if let Some(why) = silences.refused() {
        Notice::loud("The suppressions could not be read from here.")
            .saying(why.to_string())
            .saying(
                "The agent goes on silencing what it read at start-up; the summary lists it. \
                 The files are root's alone, so the console reads them as root.",
            )
            .render(look, table, buffer);
        return Vec::new();
    }
    if rows.is_empty() {
        Notice::plain("Nothing is silenced on this host.")
            .saying(
                "Every finding the agent raises reaches the console. Press d on a finding to \
                 silence the object it is about, with a reason.",
            )
            .render(look, table, buffer);
        return Vec::new();
    }

    let shape = Shape::of(area.width);
    let widths = widths(shape);
    let columns = listing::column_widths(look, &widths, table);
    let draw = |at: usize| row(&rows[at], look, shape, &columns);
    listing::render(
        look,
        header(shape),
        listing::Rows {
            total: rows.len(),
            drawn: &draw,
        },
        &widths,
        listing::Where {
            at: silences.at(),
            focused: true,
        },
        table,
        buffer,
    )
}

fn row(shown: &Shown, look: Look, shape: Shape, columns: &[usize]) -> Row<'static> {
    let cut = |index: usize, text: &str| match columns.get(index) {
        Some(width) => column::fit(text, *width),
        None => text.to_string(),
    };
    let standing = match shown.standing {
        Standing::InForce => look.palette.quiet(),
        _ => look.palette.accent(),
    };

    let mut cells = vec![
        Cell::from(Span::styled(cut(0, shown.standing.said()), standing)),
        Cell::from(cut(1, &shown.object)),
    ];
    match shape {
        Shape::Roomy => {
            cells.push(Cell::from(cut(2, &shown.kind)));
            cells.push(Cell::from(cut(3, &shown.until)));
            cells.push(Cell::from(cut(4, &shown.reason)));
            cells.push(Cell::from(cut(5, &named(&shown.file))));
        }
        Shape::Plain => {
            cells.push(Cell::from(cut(2, &shown.kind)));
            cells.push(Cell::from(cut(3, &shown.reason)));
        }
        Shape::Cramped => cells.push(Cell::from(cut(2, &shown.reason))),
    }
    Row::new(cells)
}

fn named(file: &str) -> String {
    Path::new(file)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn tally(rows: &[Shown], at: usize, width: u16) -> String {
    let count = |standing: Standing| rows.iter().filter(|row| row.standing == standing).count();
    let mut parts = vec![format!("{} silenced", rows.len())];
    for standing in [Standing::NotReadYet, Standing::StillHeld, Standing::Unasked] {
        match count(standing) {
            0 => {}
            held => parts.push(format!("{held} {}", standing.said())),
        }
    }
    match rows.get(at) {
        Some(row) if row.written.is_some() => {
            parts.push(format!("u reports it again, taken out of {}", row.file))
        }
        Some(_) => parts.push("already out of its file".to_string()),
        None => {}
    }

    let room = (width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in parts {
        let next = match line.is_empty() {
            true => part,
            false => format!("{line} \u{b7} {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    format!(" {line}")
}
