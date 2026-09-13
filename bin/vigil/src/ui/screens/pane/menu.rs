use ratatui::text::{Line, Span};
use vigil_view::Pane;

use crate::ui::{Arrows, Look};

pub(super) fn row_of_names(
    look: Look,
    panes: &[Box<dyn Pane>],
    shown: &[usize],
    at: usize,
    arrows: Arrows,
) -> Line<'static> {
    let mut spans = vec![Span::styled(
        match arrows {
            Arrows::Menu => " ▸ ",
            _ => "   ",
        },
        look.palette.heading(),
    )];

    for (place, index) in shown.iter().enumerate() {
        let pane = &panes[*index];
        let index = *index;
        if place > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
        }
        match index == at {
            true => spans.push(Span::styled(
                format!("[{}]", pane.name()),
                match arrows {
                    Arrows::Menu => look.palette.selected(),
                    _ => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {} ", pane.name()))),
        }
    }

    Line::from(spans)
}
