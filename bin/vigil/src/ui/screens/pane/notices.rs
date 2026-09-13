use vigil_view::{Pane, time_of_day};

use crate::ui::helpers::words::gone as words_gone;
use crate::ui::helpers::words::refusal;
use crate::ui::{Gone, Notice, Reading, View};

pub(super) fn missing(view: &View, pane: &dyn Pane) -> Option<Notice> {
    let collector = pane.reads();
    if view.switched_off(collector) {
        return Some(
            Notice::plain(format!("This agent is not reading {collector}."))
                .saying(
                    view.collector_reason(collector)
                        .unwrap_or("switched off in the configuration")
                        .to_string(),
                )
                .saying("The console asks for no reading it was told is switched off."),
        );
    }

    match view.reading(collector) {
        Reading::Unknown => Some(
            Notice::plain("The agent has not been asked yet.")
                .saying("Press r to ask now; otherwise every couple of seconds."),
        ),
        Reading::NotTakenYet => Some(
            Notice::plain(format!("The agent has not read {collector} yet."))
                .saying("The first reading is due within the interval on the summary screen."),
        ),
        Reading::Refused(refused) => Some(refusal::refused(
            refused,
            collector,
            pane.nothing_was_read(),
        )),
        Reading::Taken(_) => None,
    }
}

pub(super) fn said(notice: vigil_view::Notice) -> Notice {
    let mut said = match notice.loud {
        true => Notice::loud(notice.headline),
        false => Notice::plain(notice.headline),
    };
    for sentence in notice.detail {
        said = said.saying(sentence);
    }
    said
}

pub(super) fn nothing_here(pane: &dyn Pane, taken_at: &str) -> Notice {
    match pane.nothing_in_the_reading() {
        Some(notice) => said(notice),
        None => Notice::plain(format!(
            "The reading taken at {} found nothing.",
            time_of_day(taken_at)
        ))
        .saying("The reading did not fail."),
    }
}

pub(super) fn gone(gone: &Gone) -> Notice {
    words_gone::out_of_the_reading(gone)
}
