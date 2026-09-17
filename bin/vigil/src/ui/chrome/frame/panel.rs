use ratatui::text::Line;
use ratatui::widgets::Block;

use super::header::standing;
use crate::ui::theme::caption::Keys;
use crate::ui::theme::panel;
use crate::ui::{Look, Screen, View};

pub(super) fn panel(look: Look, screen: Screen, view: &View) -> Block<'static> {
    let block = panel::block(look, screen.title(), Keys::Here, true);

    match (view.stale(), standing(look, view)) {
        (Some(_), Some((tail, style))) => block
            .border_style(look.palette.alarm())
            .title_top(Line::styled(format!(" {tail} "), style).right_aligned()),
        (None, Some((tail, _))) => panel::tail(block, look, &tail),
        (_, None) => block,
    }
}
