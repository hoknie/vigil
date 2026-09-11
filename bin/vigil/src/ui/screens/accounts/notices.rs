use super::showing::Showing;
use crate::ui::helpers::words::{moment, refusal};
use crate::ui::{Notice, Reading, Subject, View};

pub(super) fn missing(view: &View) -> Option<Notice> {
    match view.reading("users") {
        Reading::Unknown => {
            Some(Notice::plain("The agent has not been asked yet.").saying("Press r to ask now."))
        }
        Reading::NotTakenYet => Some(
            Notice::plain("The agent has not read the accounts yet.")
                .saying("The first reading is due within the interval on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(refusal::refused(
            refusal,
            "users",
            "No account is listed here: nothing was read.",
        )),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::loud(format!(
                "The reading taken at {} holds no account at all.",
                moment::time_of_day(&snapshot.taken_at)
            ))
            .saying(
                "Every host has accounts. Treat this as a failed reading: the summary \
                 screen carries the collector's state.",
            ),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(view: &View, showing: &Showing<'_>) -> Notice {
    let subject = showing.subject;

    if showing.search.holding_back() {
        return Notice::plain(format!(
            "No {} matches {:?}.",
            subject.thing(),
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match view.collector_note("users") {
        Some(reason) => {
            Notice::loud("Nothing here, and the reading is incomplete.").saying(format!(
                "Reason: {reason}. It may or may not be about this list; the summary screen \
                 carries the whole of it."
            ))
        }
        None => Notice::plain(format!("No {} in this reading.", subject.thing())).saying(
            match subject {
                Subject::LoggedIn => {
                    "Nobody is logged in, by this host's login records. That is the only \
                     source read."
                }
                Subject::Keys | Subject::SshUsers => {
                    "No authorized_keys file was found for any account the agent could look at."
                }
                _ => "This reading was empty.",
            },
        ),
    }
}
