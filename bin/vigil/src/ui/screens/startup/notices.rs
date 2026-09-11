use super::rows::COLLECTOR;
use super::showing::Showing;
use crate::ui::helpers::words::{moment, refusal};
use crate::ui::{Notice, Reading, View};

pub(super) fn missing(view: &View) -> Option<Notice> {
    if view.switched_off(COLLECTOR) {
        return Some(
            Notice::plain("This agent is not reading what the host starts by itself.")
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
            Notice::plain("The agent has not read what starts by itself yet.")
                .saying("The first reading is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(refusal::refused(
            refusal,
            COLLECTOR,
            "Nothing the host starts by itself is listed here: nothing was read.",
        )),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => {
            let mut notice = Notice::loud(format!(
                "The reading taken at {} holds nothing at all.",
                moment::time_of_day(&snapshot.taken_at)
            ))
            .saying(
                "Every host starts something. Treat this as a failed reading: the summary \
                 screen carries the collector's state.",
            );
            if let Some(reason) = view.collector_note(COLLECTOR) {
                notice = notice.saying(format!("The agent said: {reason}."));
            }
            Some(notice)
        }
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(view: &View, showing: &Showing<'_>) -> Notice {
    let list = showing.list;

    if showing.search.holding_back() {
        return Notice::plain(format!(
            "No {} matches {:?}.",
            list.thing(),
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match view.collector_note(COLLECTOR) {
        Some(reason) => {
            Notice::loud("Nothing here, and the reading is incomplete.").saying(format!(
                "Reason: {reason}. It may or may not be about this list; the summary screen \
                 carries the whole of it."
            ))
        }
        None => Notice::plain(format!("No {} in this reading.", list.thing())).saying(list.empty()),
    }
}
