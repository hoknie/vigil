use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::chrome::popup;
use crate::ui::helpers::layout::popup as placing;
use crate::ui::{Aim, Editing, Look};

pub const FOOTING: &str = " Space toggles \u{b7} Enter or Esc closes, toggles kept ";

const TALLEST: usize = 8;

const NARROWEST: usize = 24;

const AROUND: usize = 4;

pub fn render(
    editing: &Editing,
    look: Look,
    bound: Rect,
    field: Rect,
    buffer: &mut Buffer,
) -> Vec<(Aim, Rect)> {
    let Some(dropdown) = editing.dropdown() else {
        return Vec::new();
    };
    let choices = editing.choices();
    let shown = dropdown.shown(choices);
    let rows: Vec<String> = shown
        .iter()
        .map(|at| {
            let one = &choices[*at];
            match one.chosen {
                true => format!(" [x] {}", one.name),
                false => format!(" [ ] {}", one.name),
            }
        })
        .collect();
    let title = match dropdown.narrowing().is_empty() {
        true => " type to narrow ".to_string(),
        false => format!(" narrowed to {} ", dropdown.narrowing()),
    };

    let widest = rows
        .iter()
        .map(|row| row.chars().count() + 1)
        .chain([FOOTING.chars().count(), title.chars().count(), NARROWEST])
        .max()
        .unwrap_or(NARROWEST);
    let area = placing::beside(
        bound,
        field.x,
        field.y,
        (widest + AROUND) as u16,
        (rows.len().clamp(1, TALLEST) + 2) as u16,
    );
    if area.height < 3 || area.width < 3 {
        return Vec::new();
    }

    let inner = popup::frame::render(
        look,
        area,
        Line::styled(title, look.palette.heading()),
        Some(Line::styled(FOOTING, look.palette.quiet())),
        buffer,
    );
    if rows.is_empty() {
        Paragraph::new(Line::styled(
            format!("  nothing matches {}", dropdown.narrowing()),
            look.palette.quiet(),
        ))
        .render(inner, buffer);
        return vec![(Aim::Popup, area)];
    }
    let mut aims = vec![(Aim::Popup, area)];
    aims.extend(
        popup::list::render(
            look,
            area,
            inner,
            rows.into_iter().map(Line::raw).collect(),
            dropdown.at(),
            buffer,
        )
        .into_iter()
        .map(|(at, drawn)| (Aim::Choice(shown[at]), drawn)),
    );
    aims
}
