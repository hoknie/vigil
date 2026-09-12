use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::showing::Showing;
use super::{files, host};
use crate::ui::helpers::layout::panes;
use crate::ui::{Arrows, Look, System, View};

pub fn render(view: &View, look: Look, showing: &Showing<'_>, area: Rect, buffer: &mut Buffer) {
    let (menu, rest) = panes::menu(area);
    if let Some(menu) = menu {
        Paragraph::new(row_of_names(look, showing)).render(menu, buffer);
    }

    match showing.showing {
        System::Host => host::render(view, look, showing, rest, buffer),
        System::Files => files::render(view, look, showing, rest, buffer),
    }
}

fn row_of_names(look: Look, showing: &Showing<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match showing.arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (index, half) in System::ALL.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match *half == showing.showing {
            true => spans.push(Span::styled(
                format!("[{}]", half.name()),
                match showing.arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::styled(
                format!(" {} ", half.name()),
                look.palette.quiet(),
            )),
        }
    }

    Line::from(spans)
}
