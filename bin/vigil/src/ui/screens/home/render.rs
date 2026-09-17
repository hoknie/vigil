use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::columns::{
    HEALTH, HOLDS, MARKER, NUMBER, OBJECTS, READ, ROOM_FOR_THE_COLLECTOR, SECTION, STATE, widths,
};
use super::row::Row;
use super::rows::rows;
use super::tally::tally;
use crate::ui::helpers::layout::scroll;
use crate::ui::helpers::layout::{column, footing};
use crate::ui::{Arrows, Group, Look, View};

pub struct Showing {
    pub cursor: usize,
    pub arrows: Arrows,
}

const UNWELL: &str = "! ";

const WELL: &str = "  ";

pub fn render(
    view: &View,
    look: Look,
    showing: &Showing,
    area: Rect,
    buffer: &mut Buffer,
) -> Vec<(usize, Rect)> {
    if area.height < 2 {
        return Vec::new();
    }
    let rows = rows(view);
    let wide = area.width >= ROOM_FOR_THE_COLLECTOR;
    let longest = longest(&rows);
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
    let mut placed = Vec::new();
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
                    " ".repeat(MARKER + HEALTH + NUMBER),
                    column::columns(&widths(wide, longest))
                ),
                look.palette.quiet(),
            ));
        }
        for row in of_this_group {
            if at == showing.cursor {
                selected = lines.len();
            }
            placed.push((at, lines.len()));
            lines.push(drawn(
                row,
                look,
                showing,
                at == showing.cursor,
                wide,
                longest,
            ));
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

    footing::render(look, tally(&rows, footer.width), footer, buffer);
    placed
        .into_iter()
        .filter(|(_, line)| (top..top + height).contains(line))
        .map(|(at, line)| {
            (
                at,
                Rect::new(page.x, page.y + (line - top) as u16, page.width, 1),
            )
        })
        .collect()
}

pub fn printed_height(view: &View, width: u16) -> u16 {
    let rows = rows(view);
    let groups = Group::ALL
        .iter()
        .filter(|group| rows.iter().any(|row| row.group() == **group))
        .count() as u16;
    let _ = width;

    rows.len() as u16 + groups * 2 + 1 + 1
}

fn longest(rows: &[Row]) -> usize {
    rows.iter()
        .map(|row| row.name.chars().count())
        .max()
        .unwrap_or_default()
}

fn drawn(
    row: &Row,
    look: Look,
    showing: &Showing,
    here: bool,
    wide: bool,
    longest: usize,
) -> Line<'static> {
    let marker = match (here, showing.arrows == Arrows::List) {
        (true, true) => " \u{25b8} ",
        (true, false) => " · ",
        (false, _) => "   ",
    };
    let health = match row.standing.unwell {
        true => UNWELL,
        false => WELL,
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

    let cells: Vec<(&str, usize)> = widths(wide, longest)
        .into_iter()
        .map(|(heading, width)| {
            (
                match heading {
                    SECTION => row.name.as_str(),
                    HOLDS => row.holds.as_str(),
                    STATE => row.standing.state.as_str(),
                    OBJECTS => objects.as_str(),
                    READ => read.as_str(),
                    _ => row.collector.as_str(),
                },
                width,
            )
        })
        .collect();

    Line::from(vec![
        Span::styled(
            marker.to_string(),
            match here {
                true => look.palette.selected(),
                false => look.palette.heading(),
            },
        ),
        Span::styled(
            health.to_string(),
            match row.standing.unwell {
                true => look.palette.alarm(),
                false => look.palette.heading(),
            },
        ),
        Span::styled(
            number,
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
