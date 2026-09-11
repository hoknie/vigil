use ratatui::text::Line;

use crate::ui::{Filter, Look, Notice, View};

pub(super) fn notice(view: &View, filter: &Filter) -> Notice {
    match (&view.found.refused, filter.holding_back()) {
        (Some(reason), _) => Notice::loud("The findings were refused.").saying(reason.clone()),
        (None, false) => Notice::plain("Nothing has been reported since the agent started.")
            .saying("Nothing is filtered out: this is everything raised."),
        (None, true) => Notice::plain(format!("Nothing here is {}.", filter.describe())).saying(
            "Filtered here, not by the agent: press s to lower the floor, / to change the \
             search.",
        ),
    }
}

pub(super) fn searching(filter: &Filter, look: Look) -> Line<'static> {
    match filter.search().typing() || filter.search().holding_back() {
        true => filter.search().line(look),
        false => Line::styled(
            format!(" showing {}", filter.describe()),
            look.palette.heading(),
        ),
    }
}
