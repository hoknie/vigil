use vigil_view::time_of_day;

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::ui::{Look, Screen, View};

pub(super) fn standing(look: Look, view: &View) -> Option<(String, Style)> {
    match (view.stale(), view.as_of()) {
        (Some(_), Some(when)) => Some((
            format!("NOT ANSWERING · reading from {}", time_of_day(when)),
            look.palette.alarm(),
        )),
        (Some(_), None) => Some(("NOT ANSWERING".to_string(), look.palette.alarm())),
        (None, Some(when)) => Some((format!("as of {}", time_of_day(when)), look.palette.quiet())),
        (None, None) => None,
    }
}

pub(super) fn header(look: Look, screen: Screen, view: &View, width: u16) -> Line<'static> {
    let name = format!(" {} ", screen.title());
    let mut spans = vec![Span::styled(name.clone(), look.palette.heading())];
    if let Some((tail, style)) = standing(look, view) {
        let tail = format!(" {tail} ");
        let used = name.chars().count() + tail.chars().count();
        if used <= width as usize {
            spans.push(Span::raw(" ".repeat(width as usize - used)));
            spans.push(Span::styled(tail, style));
        }
    }
    Line::from(spans)
}
