use super::rows::COLLECTOR;
use super::showing::Showing;
use crate::ui::helpers::words::refusal;
use crate::ui::{Notice, Reading, View};

const NOT_THE_SAME: &str = "What is running in containers here is unknown, which is not the same as a host running \
     none.";

pub(super) fn missing(view: &View) -> Option<Notice> {
    if view.switched_off(COLLECTOR) {
        return Some(
            Notice::plain("This agent is not reading what runs in containers.")
                .saying(
                    view.collector_reason(COLLECTOR)
                        .unwrap_or("switched off in the configuration")
                        .to_string(),
                )
                .saying("The console asks for no reading it was told is switched off."),
        );
    }

    match view.reading(COLLECTOR) {
        Reading::Unknown => Some(
            Notice::plain("The agent has not been asked yet.")
                .saying("Press r to ask now; otherwise every couple of seconds."),
        ),
        Reading::NotTakenYet => Some(
            Notice::plain("The agent has not read the containers yet.")
                .saying("The first reading is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(
            refusal::refused(
                refusal,
                COLLECTOR,
                "No container and no runtime socket is listed here: nothing was read.",
            )
            .saying(NOT_THE_SAME),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    if showing.search.holding_back() {
        return Notice::plain(format!(
            "No container and no socket matches {:?}.",
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        );
    }

    Notice::plain("Nothing is running in a container on this host.").saying(
        "The reading was taken and holds no container and no runtime socket: this is a host \
         with none, not a reading that failed.",
    )
}
