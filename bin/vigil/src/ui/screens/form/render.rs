use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::field::{self, MARGIN};
use super::{buttons, dropdown};
use crate::ui::helpers::layout::wrap;
use crate::ui::{Aim, Editing, Look, Spot};

const WIDEST_LABEL: usize = 16;

const ELSEWHERE: &str = "   ";

type Place = (Spot, usize, u16, u16);

pub fn render(
    editing: &Editing,
    look: Look,
    area: Rect,
    buffer: &mut Buffer,
) -> (Option<Position>, Vec<(Aim, Rect)>) {
    let page = area.height as usize;
    if page == 0 {
        return (None, Vec::new());
    }
    let (lines, focused, places, cursor) = lines(editing, look, area.width as usize);
    let top = focused
        .saturating_sub(page - 1)
        .min(lines.len().saturating_sub(page));

    Paragraph::new(
        lines
            .into_iter()
            .skip(top)
            .take(page)
            .collect::<Vec<Line<'static>>>(),
    )
    .render(area, buffer);

    let mut aims = Vec::new();
    let mut field_drawn = None;
    for (spot, line, x, width) in places {
        if line < top || line >= top + page {
            continue;
        }
        let drawn = Rect {
            x: area.x + x.min(area.width),
            y: area.y + (line - top) as u16,
            width: width.min(area.width.saturating_sub(x)),
            height: 1,
        };
        if spot == editing.spot() {
            field_drawn = Some(drawn);
        }
        aims.push((Aim::Spot(spot), drawn));
    }

    let cursor = match (cursor, field_drawn, editing.dropdown()) {
        (Some(column), Some(drawn), None) => Some(Position {
            x: (area.x + column).min(area.right().saturating_sub(1)),
            y: drawn.y,
        }),
        _ => None,
    };
    if let Some(drawn) = field_drawn {
        aims.extend(dropdown::render(editing, look, area, drawn, buffer));
    }
    (cursor, aims)
}

type Laid = (Vec<Line<'static>>, usize, Vec<Place>, Option<u16>);

fn lines(editing: &Editing, look: Look, width: usize) -> Laid {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut places: Vec<Place> = Vec::new();
    let mut focused = 0;
    let mut cursor = None;
    let room = width.saturating_sub(MARGIN + 1).max(1);

    let (back, placed) = buttons::line(
        &[("\u{2190} Back", Spot::Back, editing.spot() == Spot::Back)],
        look,
    );
    places.extend(placed.into_iter().map(|(spot, x, wide)| (spot, 0, x, wide)));
    lines.push(back);
    lines.push(Line::raw(""));
    for part in wrap::wrap(&editing.form().caption, room) {
        lines.push(Line::styled(
            format!("{ELSEWHERE}{part}"),
            look.palette.heading(),
        ));
    }
    for said in &editing.form().about {
        for part in wrap::wrap(said, room) {
            lines.push(Line::styled(
                format!("{ELSEWHERE}{part}"),
                look.palette.quiet(),
            ));
        }
    }
    lines.push(Line::raw(""));

    let label = editing
        .form()
        .fields
        .iter()
        .map(|field| field.label.chars().count())
        .max()
        .unwrap_or(0)
        .min(WIDEST_LABEL)
        .min(width / 3);
    let input = editing.input();
    for (at, one) in editing.form().fields.iter().enumerate() {
        let here = editing.spot() == Spot::Field(at);
        if here {
            focused = lines.len();
        }
        let (drawn, x, wide, column) = field::one(one, here, input.as_ref(), label, look, width);
        if one.editable() {
            places.push((Spot::Field(at), lines.len(), x, wide));
        }
        if here {
            cursor = column;
        }
        lines.extend(drawn);
    }

    lines.push(Line::raw(""));
    if let Some(trouble) = editing.trouble() {
        for part in wrap::wrap(trouble, room) {
            lines.push(Line::styled(
                format!("{ELSEWHERE}{part}"),
                look.palette.alarm(),
            ));
        }
        lines.push(Line::raw(""));
    }
    if matches!(editing.spot(), Spot::Save | Spot::Cancel) {
        focused = lines.len();
    }
    let (end, placed) = buttons::line(
        &[
            ("Save", Spot::Save, editing.spot() == Spot::Save),
            ("Cancel", Spot::Cancel, editing.spot() == Spot::Cancel),
        ],
        look,
    );
    let at = lines.len();
    places.extend(
        placed
            .into_iter()
            .map(|(spot, x, wide)| (spot, at, x, wide)),
    );
    lines.push(end);

    (lines, focused, places, cursor)
}
