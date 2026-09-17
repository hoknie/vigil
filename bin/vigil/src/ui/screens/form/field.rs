use ratatui::style::Style;
use ratatui::text::{Line, Span};
use tui_input::Input;
use vigil_view::{Entry, Field};

use super::inset::{self, EDGES, LEFT, RIGHT};
use crate::ui::Look;
use crate::ui::helpers::layout::{column, wrap};

pub const MARGIN: usize = 3;

const WIDEST_FIELD: usize = 48;

const HERE: &str = " \u{25b8} ";

const ELSEWHERE: &str = "   ";

pub type Placed = (Vec<Line<'static>>, u16, u16, Option<u16>);

pub fn one(
    field: &Field,
    here: bool,
    input: Option<&Input>,
    label: usize,
    look: Look,
    width: usize,
) -> Placed {
    let lead = match here {
        true => HERE,
        false => ELSEWHERE,
    };
    let start = MARGIN + label + 2;
    let indent = " ".repeat(start);
    let room = width.saturating_sub(start + 1).max(8);
    let boxed = room.min(WIDEST_FIELD);
    let inside = boxed.saturating_sub(EDGES).max(1);
    let mut cursor = None;

    let values: Vec<Vec<Span<'static>>> = match &field.entry {
        Entry::Fixed(value) => wrap::wrap(value, room)
            .into_iter()
            .map(|part| vec![Span::raw(part)])
            .collect(),
        Entry::Text(value) => {
            let scroll = match (here, input) {
                (true, Some(input)) => {
                    let scroll = input.visual_scroll(inside.saturating_sub(1));
                    cursor = Some((start + 2 + input.visual_cursor() - scroll) as u16);
                    scroll
                }
                _ => 0,
            };
            vec![framed(inset::visible(value, scroll, inside), look)]
        }
        Entry::Switch(on) => vec![vec![Span::styled(
            match on {
                true => "[x] yes",
                false => "[ ] no",
            },
            match here {
                true => look.palette.selected(),
                false => Style::default(),
            },
        )]],
        Entry::Choices(choices) if choices.is_empty() => {
            vec![vec![Span::raw("nothing to choose from")]]
        }
        entry @ Entry::Choices(_) => {
            vec![framed(inset::summary(&entry.chosen(), inside), look)]
        }
    };
    let wide = match &field.entry {
        Entry::Text(_) | Entry::Choices(_) => (inside + EDGES) as u16,
        _ => room as u16,
    };

    let mut lines = Vec::new();
    for (at, value) in values.into_iter().enumerate() {
        let opening = match at {
            0 => format!(
                "{lead}{}  ",
                column::fit(&field.label, label).trim_end().to_string()
                    + &" ".repeat(label.saturating_sub(field.label.chars().count().min(label)))
            ),
            _ => indent.clone(),
        };
        let mut spans = vec![Span::styled(opening, look.palette.label())];
        spans.extend(value);
        lines.push(Line::from(spans));
    }
    if let Some(hint) = &field.hint {
        for part in wrap::wrap(hint, room) {
            lines.push(Line::styled(
                format!("{indent}{part}"),
                look.palette.quiet(),
            ));
        }
    }
    (lines, start as u16, wide, cursor)
}

fn framed(inside: String, look: Look) -> Vec<Span<'static>> {
    vec![
        Span::styled(LEFT, look.palette.border()),
        Span::styled(format!(" {inside} "), look.palette.field()),
        Span::styled(RIGHT, look.palette.border()),
    ]
}
