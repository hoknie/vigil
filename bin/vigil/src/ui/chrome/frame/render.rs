use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::header::header;
use super::hints::Hints;
use super::keys::keys;
use super::panel::panel;
use super::status::{beside, status};
use super::title::title;
use crate::ui::{Look, Screen, View};

pub fn body(look: Look, area: Rect, panes: bool) -> Rect {
    if !room_for_a_frame(area) {
        return area;
    }
    if !look.interactive() {
        return Rect {
            y: area.y + 3,
            height: area.height - 3,
            ..area
        };
    }
    match panes {
        true => Rect {
            y: area.y + 2,
            height: area.height.saturating_sub(4),
            ..area
        },
        false => Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(5),
        },
    }
}

fn room_for_a_frame(area: Rect) -> bool {
    area.height >= 8 && area.width >= 24
}

pub fn render(
    look: Look,
    screen: Screen,
    view: &View,
    hints: Hints<'_>,
    area: Rect,
    buffer: &mut Buffer,
) -> Rect {
    if !room_for_a_frame(area) {
        return area;
    }

    let band = |y: u16| Rect {
        y: area.y + y,
        height: 1,
        ..area
    };

    if !look.interactive() {
        title(look, view).render(band(0), buffer);
        Paragraph::new(Line::styled(status(view, area.width), look.palette.quiet()))
            .render(band(1), buffer);
        Paragraph::new(Line::styled(
            "─".repeat(area.width as usize),
            look.palette.border(),
        ))
        .render(band(2), buffer);
        return Rect {
            y: area.y + 3,
            height: area.height - 3,
            ..area
        };
    }

    title(look, view).render(band(0), buffer);
    let room = match hints.mouse {
        Some(said) => area.width.saturating_sub(said.chars().count() as u16 + 1),
        None => area.width,
    };
    Paragraph::new(Line::styled(
        beside(status(view, room), hints.mouse, area.width),
        look.palette.quiet(),
    ))
    .render(band(area.height - 2), buffer);
    match (hints.message, hints.asking) {
        (Some(message), _) => {
            Paragraph::new(Line::styled(format!(" {message}"), look.palette.alarm()))
                .render(band(area.height - 1), buffer)
        }
        (None, Some(asking)) => {
            Paragraph::new(Line::styled(asking.to_string(), look.palette.heading()))
                .render(band(area.height - 1), buffer)
        }
        (None, None) => Paragraph::new(Line::styled(
            keys(&hints, screen, area.width),
            look.palette.quiet(),
        ))
        .render(band(area.height - 1), buffer),
    }

    match hints.panes {
        true => Paragraph::new(header(look, screen, view, area.width)).render(band(1), buffer),
        false => panel(look, screen, view).render(
            Rect {
                y: area.y + 1,
                height: area.height - 3,
                ..area
            },
            buffer,
        ),
    }
    body(look, area, hints.panes)
}
