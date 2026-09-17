use ratatui::text::{Line, Span};

use crate::ui::{Look, Spot};

pub fn line(named: &[(&str, Spot, bool)], look: Look) -> (Line<'static>, Vec<(Spot, u16, u16)>) {
    let mut spans = vec![Span::raw(" ")];
    let mut places = Vec::new();
    let mut x = 1u16;
    for (name, spot, here) in named {
        let (lead, style) = match here {
            true => ("\u{25b8}", look.palette.selected()),
            false => (" ", look.palette.accent()),
        };
        let drawn = format!("[ {name} ]");
        let wide = drawn.chars().count() as u16;
        places.push((*spot, x + 1, wide));
        x += 1 + wide + 2;
        spans.push(Span::raw(lead.to_string()));
        spans.push(Span::styled(drawn, style));
        spans.push(Span::raw("  "));
    }
    (Line::from(spans), places)
}
