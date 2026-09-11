use ratatui::text::Line;

use crate::ui::helpers::layout::wrap;

pub(super) fn bullet(sentence: &str, width: usize) -> Vec<Line<'static>> {
    wrap::wrap(sentence, width.saturating_sub(5))
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            Line::raw(match index {
                0 => format!("   · {part}"),
                _ => format!("     {part}"),
            })
        })
        .collect()
}

pub(super) fn note(text: &str, width: usize) -> Vec<Line<'static>> {
    wrap::wrap(text, width.saturating_sub(7))
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            Line::raw(match index {
                0 => format!("     ↳ {part}"),
                _ => format!("       {part}"),
            })
        })
        .collect()
}
