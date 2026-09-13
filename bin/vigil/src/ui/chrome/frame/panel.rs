use vigil_view::time_of_day;

use ratatui::layout::Alignment;
use ratatui::text::Line;
use ratatui::widgets::Block;

use crate::ui::{Look, Screen, View};

pub(super) fn panel(look: Look, screen: Screen, view: &View, focused: bool) -> Block<'static> {
    let block = Block::bordered()
        .border_style(match view.stale().is_some() {
            true => look.palette.alarm(),
            false => look.palette.border(),
        })
        .title(Line::styled(
            match focused {
                true => format!(" ▸ {} ", screen.title()),
                false => format!("   {} ", screen.title()),
            },
            look.palette.heading(),
        ));

    match (view.stale(), view.as_of()) {
        (Some(_), Some(when)) => block.title(
            Line::styled(
                format!(" NOT ANSWERING · reading from {} ", time_of_day(when)),
                look.palette.alarm(),
            )
            .alignment(Alignment::Right),
        ),
        (Some(_), None) => block.title(
            Line::styled(" NOT ANSWERING ", look.palette.alarm()).alignment(Alignment::Right),
        ),
        (None, Some(when)) => block.title(
            Line::styled(
                format!(" as of {} ", time_of_day(when)),
                look.palette.quiet(),
            )
            .alignment(Alignment::Right),
        ),
        (None, None) => block,
    }
}
