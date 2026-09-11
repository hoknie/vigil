use super::showing::Showing;
use crate::ui::helpers::words::{moment, refusal};
use crate::ui::{Notice, Reading, View};

pub(super) fn missing(view: &View) -> Option<Notice> {
    match view.reading("ports") {
        Reading::Unknown => Some(
            Notice::plain("The agent has not been asked yet.")
                .saying("Press r to ask now; otherwise every couple of seconds."),
        ),
        Reading::NotTakenYet => Some(
            Notice::plain("The agent has not read the sockets yet.")
                .saying("The first reading is due within the interval on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(refusal::refused(
            refusal,
            "ports",
            "No listening socket is listed here: nothing was read.",
        )),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::plain(format!(
                "The reading taken at {} found no listening sockets.",
                moment::time_of_day(&snapshot.taken_at)
            ))
            .saying("Nothing is listening. The reading did not fail."),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    let by_kind = showing.protocols.holding_back();
    let by_word = showing.search.holding_back();

    match (by_kind, by_word) {
        (true, true) => Notice::plain(format!(
            "Nothing matching {:?} is left among the kinds you are showing.",
            showing.search.query()
        ))
        .saying("Two filters are on: press a for every kind of socket, Esc to drop both."),
        (true, false) => Notice::plain(format!(
            "No socket here is {}.",
            showing.protocols.describe()
        ))
        .saying("Filtered here, not by the agent: press a to show every kind."),
        (false, true) => Notice::plain(format!(
            "Nothing the agent read here matches {:?}.",
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about a socket: address, port, program, \
             path, command line, account. Press / to change it, Esc to drop it.",
        ),
        (false, false) => Notice::plain("Nothing is listening."),
    }
}
