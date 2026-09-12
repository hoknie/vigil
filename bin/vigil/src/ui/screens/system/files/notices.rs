use super::super::showing::Showing;
use super::rows::COLLECTOR;
use crate::ui::helpers::words::refusal;
use crate::ui::{Notice, Reading, View};

const NOT_THE_SAME: &str = "What these files hold is unknown, which is not the same as a host where none of them \
     changed.";

const NAMED_IN_THE_CONFIGURATION: &str =
    "The list is the paths named in the configuration file, plus the directories on PATH.";

pub(super) fn missing(view: &View) -> Option<Notice> {
    if view.switched_off(COLLECTOR) {
        return Some(
            Notice::plain("This agent is not watching any file.")
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
            Notice::plain("The agent has not read the watched files yet.")
                .saying("The first reading is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(
            refusal::refused(
                refusal,
                COLLECTOR,
                "No file and no directory is listed here: nothing was read.",
            )
            .saying(NAMED_IN_THE_CONFIGURATION)
            .saying(NOT_THE_SAME),
        ),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::plain("No path is being watched on this host.")
                .saying(NAMED_IN_THE_CONFIGURATION)
                .saying(
                    "A host with nothing named watches nothing: this is a configuration that \
                     asked for none, not a reading that failed.",
                ),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    match showing.search.holding_back() {
        true => Notice::plain(format!(
            "No file and no directory matches {:?}.",
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        ),
        false => Notice::plain("This reading lists no file and no directory.")
            .saying(NAMED_IN_THE_CONFIGURATION),
    }
}
