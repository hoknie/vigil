use super::super::showing::Showing;
use super::rows::COLLECTOR;
use crate::ui::helpers::words::refusal;
use crate::ui::{Notice, Reading, View};

const NOT_THE_SAME: &str = "What this host is running on is unknown, which is not the same as a host with room to \
     spare.";

pub(super) fn missing(view: &View) -> Option<Notice> {
    if view.switched_off(COLLECTOR) {
        return Some(
            Notice::plain("This agent is not reading what the host runs on.")
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
            Notice::plain("The agent has not read what the host runs on yet.")
                .saying("The first reading is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(
            refusal::refused(
                refusal,
                COLLECTOR,
                "No boot, no memory and no filesystem is listed here: nothing was read.",
            )
            .saying(NOT_THE_SAME),
        ),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::loud("This reading holds nothing at all.")
                .saying(
                    "Every host has a boot and a memory, so a reading with no row at all is a \
                     failed one.",
                )
                .saying(NOT_THE_SAME),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    match showing.search.holding_back() {
        true => Notice::plain(format!(
            "No boot, memory or filesystem matches {:?}.",
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        ),
        false => Notice::plain("This reading lists nothing at all.").saying(NOT_THE_SAME),
    }
}
