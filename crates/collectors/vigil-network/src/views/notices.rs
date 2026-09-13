use vigil_view::{Notice, Showing};

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    match (!showing.hidden.is_empty(), showing.holding_back()) {
        (true, true) => Notice::plain(format!(
            "Nothing matching {:?} is left among the kinds you are showing.",
            showing.search
        ))
        .saying("Two filters are on: press a for every kind of socket, Esc to drop both."),
        (true, false) => Notice::plain(format!("No socket here is {}.", shown(showing)))
            .saying("Filtered here, not by the agent: press a to show every kind."),
        (false, true) => Notice::plain(format!(
            "Nothing the agent read here matches {:?}.",
            showing.search
        ))
        .saying(
            "The search covers every value recorded about a socket: address, port, program, \
             path, command line, account. Press / to change it, Esc to drop it.",
        ),
        (false, false) => Notice::plain("Nothing is listening."),
    }
}

fn shown(showing: &Showing<'_>) -> String {
    format!("shown while {} is hidden", showing.hidden.join(", "))
}
