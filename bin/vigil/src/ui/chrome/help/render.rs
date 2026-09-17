use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};

use super::rows::rows;
use crate::ui::Look;
use crate::ui::helpers::layout::column;

const WIDEST: u16 = 78;

const KEYS: usize = 18;

pub fn render(look: Look, area: Rect, buffer: &mut Buffer) {
    let rows = rows();
    let width = WIDEST.min(area.width);
    let height = (rows.len() as u16 + 2).min(area.height);
    let panel = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    Clear.render(panel, buffer);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(look.palette.border())
        .title(Line::styled(" KEYS ", look.palette.heading()))
        .title_bottom(
            Line::styled(" any key closes this ", look.palette.quiet()).alignment(Alignment::Right),
        );
    let inside = block.inner(panel);
    block.render(panel, buffer);

    Paragraph::new(
        rows.into_iter()
            .map(|(keys, meaning)| match keys.is_empty() {
                true => Line::styled(meaning.to_string(), look.palette.heading()),
                false => Line::from(vec![
                    Span::styled(
                        format!("  {}", column::fit(keys, KEYS)),
                        look.palette.accent(),
                    ),
                    Span::raw(meaning.to_string()),
                ]),
            })
            .collect::<Vec<Line>>(),
    )
    .render(inside, buffer);
}
