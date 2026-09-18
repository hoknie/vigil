use ratatui::text::Line;

use super::tally::journal;
use crate::ui::{Filter, Look, Notice, View};

pub(super) fn notice(view: &View, filter: &Filter, hidden: usize) -> Notice {
    match (&view.found.refused, filter.holding_back(), hidden) {
        (Some(reason), _, _) => Notice::loud("The findings were refused.").saying(reason.clone()),
        (None, _, hidden) if hidden > 0 => {
            Notice::plain("Nothing is on this screen: what the agent raised is silenced from here.")
                .saying(format!(
                    "{hidden} row(s) hidden here; the agent stops raising them on its next round. \
                     Press u to take the entry out again."
                ))
        }
        (None, true, _) => Notice::plain(format!("Nothing here is {}.", filter.describe())).saying(
            "Filtered here, not by the agent: press f to lower the floor, / to change the \
             search.",
        ),
        (None, false, _) => nothing_open(view),
    }
}

fn nothing_open(view: &View) -> Notice {
    let notice = Notice::plain("Nothing is on this screen, and nothing is filtered out.");

    let Some(store) = journal(view) else {
        return notice.saying(
            "Nothing has been raised since this agent started. This agent does not say what \
             its local history holds.",
        );
    };

    let notice = match (store.records.held, open(view)) {
        (0, _) => notice.saying("No finding has been written to the journal yet."),
        (records, 0) => notice.saying(format!(
            "The journal holds {records} record(s), and none of them is open."
        )),
        (records, still_open) => notice.saying(format!(
            "The journal holds {records} record(s), {still_open} of them open, and none of \
             those reached this screen."
        )),
    };

    let notice = match &store.journal_path {
        Some(path) => notice.saying(format!("The journal is at {path}, and `jq` reads it.")),
        None => notice.saying("The journal is in the agent's state directory, and `jq` reads it."),
    };

    match store.damaged {
        0 => notice,
        damaged => notice.saying(format!(
            "{damaged} line(s) of it could not be read when the agent started. They were \
             counted, not thrown away."
        )),
    }
}

fn open(view: &View) -> u64 {
    view.status
        .as_ref()
        .map(|status| status.agent.findings.total)
        .unwrap_or_default()
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
