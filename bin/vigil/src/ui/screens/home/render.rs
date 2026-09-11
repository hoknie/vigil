use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{MARKER, NUMBER, ROOM_FOR_THE_COLLECTOR, widths};
use super::row::Row;
use super::rows::rows;
use super::tally::tally;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::scroll;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Arrows, Group, Look, View};

pub struct Showing {
    pub cursor: usize,
    pub arrows: Arrows,
}

pub fn render(view: &View, look: Look, showing: &Showing, area: Rect, buffer: &mut Buffer) {
    if area.height < 2 {
        return;
    }
    let rows = rows(view);
    let wide = area.width >= ROOM_FOR_THE_COLLECTOR;
    let (page, footer) = (
        Rect {
            height: area.height - 1,
            ..area
        },
        Rect {
            y: area.y + area.height - 1,
            height: 1,
            ..area
        },
    );

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut at = 0usize;
    let mut selected = 0usize;
    for group in Group::ALL {
        let of_this_group: Vec<&Row> = rows.iter().filter(|row| row.group() == *group).collect();
        if of_this_group.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }
        lines.push(Line::styled(
            format!("   {}", group.caption()),
            look.palette.heading(),
        ));
        if *group == Group::Reads {
            lines.push(Line::styled(
                format!(
                    "{}{}",
                    " ".repeat(MARKER + NUMBER),
                    column::columns(&widths(wide))
                ),
                look.palette.quiet(),
            ));
        }
        for row in of_this_group {
            if at == showing.cursor {
                selected = lines.len();
            }
            lines.push(drawn(row, look, showing, at == showing.cursor, wide));
            for note in note(row, look, area.width as usize) {
                lines.push(note);
            }
            at += 1;
        }
    }

    let height = page.height as usize;
    let top = scroll::window(selected, lines.len(), height);
    Paragraph::new(
        lines
            .iter()
            .skip(top)
            .take(height)
            .cloned()
            .collect::<Vec<Line>>(),
    )
    .render(page, buffer);

    Paragraph::new(Line::styled(
        tally(&rows, footer.width),
        look.palette.quiet(),
    ))
    .render(footer, buffer);
}

fn drawn(row: &Row, look: Look, showing: &Showing, here: bool, wide: bool) -> Line<'static> {
    let marker = match (here, showing.arrows == Arrows::List, row.standing.unwell) {
        (true, true, _) => " > ",
        (true, false, _) => " · ",
        (false, _, true) => " ! ",
        (false, _, false) => "   ",
    };
    let number = match row.number {
        Some(number) => format!("{number} "),
        None => " ".repeat(NUMBER),
    };
    let objects = match row.standing.objects {
        Some(count) => count.to_string(),
        None => "—".to_string(),
    };
    let read = row.standing.read.clone().unwrap_or_else(|| "—".to_string());

    let mut cells = vec![
        (row.name.as_str(), 9),
        (row.holds.as_str(), 26),
        (row.standing.state.as_str(), 11),
        (objects.as_str(), 7),
        (read.as_str(), 8),
    ];
    if wide {
        cells.push((row.collector.as_str(), 12));
    }

    Line::from(vec![
        Span::styled(
            format!("{marker}{number}"),
            match here {
                true => look.palette.selected(),
                false => look.palette.heading(),
            },
        ),
        match here {
            true => Span::styled(
                column::columns(&cells).trim_end().to_string(),
                look.palette.selected(),
            ),
            false => Span::raw(column::columns(&cells).trim_end().to_string()),
        },
    ])
}

fn note(row: &Row, look: Look, width: usize) -> Vec<Line<'static>> {
    let Some(note) = &row.standing.note else {
        return Vec::new();
    };
    wrap::wrap(note, width.saturating_sub(8))
        .into_iter()
        .map(|part| {
            Line::styled(
                format!("       {part}"),
                match row.standing.unwell {
                    true => look.palette.alarm(),
                    false => look.palette.quiet(),
                },
            )
        })
        .collect()
}
