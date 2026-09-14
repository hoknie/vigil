use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use vigil_view::{Entry, Field};

use crate::ui::helpers::layout::{column, wrap};
use crate::ui::{Editing, Look, Spot};

const WIDEST_LABEL: usize = 16;

const MARGIN: usize = 3;

const CARET: char = '\u{2582}';

const HERE: &str = " \u{25b8} ";

const ELSEWHERE: &str = "   ";

pub fn render(editing: &Editing, look: Look, area: Rect, buffer: &mut Buffer) {
    let page = area.height as usize;
    if page == 0 {
        return;
    }
    let (lines, focused) = lines(editing, look, area.width as usize);
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
}

pub fn lines(editing: &Editing, look: Look, width: usize) -> (Vec<Line<'static>>, usize) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut focused = 0;
    let room = width.saturating_sub(MARGIN + 1).max(1);

    lines.push(buttons(
        &[("\u{2190} Back", editing.spot() == Spot::Back)],
        look,
    ));
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
    for (at, field) in editing.form().fields.iter().enumerate() {
        let here = editing.spot() == Spot::Field(at);
        if here {
            focused = lines.len();
        }
        lines.extend(one(field, here, editing.choice(), label, look, width));
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
    lines.push(buttons(
        &[
            ("Save", editing.spot() == Spot::Save),
            ("Cancel", editing.spot() == Spot::Cancel),
        ],
        look,
    ));

    (lines, focused)
}

fn buttons(named: &[(&str, bool)], look: Look) -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    for (name, here) in named {
        let (lead, style) = match here {
            true => ("\u{25b8}", look.palette.selected()),
            false => (" ", look.palette.accent()),
        };
        spans.push(Span::raw(lead.to_string()));
        spans.push(Span::styled(format!("[ {name} ]"), style));
        spans.push(Span::raw("  "));
    }
    Line::from(spans)
}

fn one(
    field: &Field,
    here: bool,
    choice: usize,
    label: usize,
    look: Look,
    width: usize,
) -> Vec<Line<'static>> {
    let lead = match here {
        true => HERE,
        false => ELSEWHERE,
    };
    let indent = " ".repeat(MARGIN + label + 2);
    let room = width.saturating_sub(MARGIN + label + 3).max(8);
    let style = match here {
        true => look.palette.selected(),
        false => Style::default(),
    };

    let values: Vec<String> = match &field.entry {
        Entry::Fixed(value) => wrap::wrap(value, room),
        Entry::Text(value) => vec![text(value, here, room)],
        Entry::Switch(on) => vec![
            match on {
                true => "[x] yes",
                false => "[ ] no",
            }
            .to_string(),
        ],
        Entry::Choices(choices) if choices.is_empty() => {
            vec!["nothing to choose from".to_string()]
        }
        Entry::Choices(choices) => laid(
            &choices
                .iter()
                .enumerate()
                .map(|(at, one)| {
                    let mark = match one.chosen {
                        true => "[x]",
                        false => "[ ]",
                    };
                    let cursor = match here && at == choice {
                        true => "\u{25b8}",
                        false => " ",
                    };
                    format!("{cursor}{mark} {}", one.name)
                })
                .collect::<Vec<String>>(),
            room,
        ),
    };

    let mut lines = Vec::new();
    for (at, value) in values.into_iter().enumerate() {
        let start = match at {
            0 => format!(
                "{lead}{}  ",
                column::fit(&field.label, label).trim_end().to_string()
                    + &" ".repeat(label.saturating_sub(field.label.chars().count().min(label)))
            ),
            _ => indent.clone(),
        };
        lines.push(Line::from(vec![
            Span::styled(start, look.palette.label()),
            Span::styled(value, style),
        ]));
    }
    if let Some(hint) = &field.hint {
        for part in wrap::wrap(hint, room) {
            lines.push(Line::styled(
                format!("{indent}{part}"),
                look.palette.quiet(),
            ));
        }
    }
    lines
}

fn text(value: &str, here: bool, room: usize) -> String {
    let inside = room.saturating_sub(4).max(1);
    let mut shown: String = value.to_string();
    if here {
        shown.push(CARET);
    }
    let count = shown.chars().count();
    if count > inside {
        shown = shown.chars().skip(count - inside).collect();
    }
    format!("[ {shown} ]")
}

fn laid(pieces: &[String], room: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for piece in pieces {
        let wanted = match line.is_empty() {
            true => piece.chars().count(),
            false => line.chars().count() + 2 + piece.chars().count(),
        };
        if !line.is_empty() && wanted > room {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push_str("  ");
        }
        line.push_str(piece);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
