use vigil_view::{Notice, Showing};

use crate::types::Subject;

pub(super) fn nothing_read() -> Notice {
    Notice::loud("The reading holds no account at all.").saying(
        "Every host has accounts. Treat this as a failed reading: the summary screen carries \
         the collector's state.",
    )
}

pub(super) fn empty(subject: Subject, showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!(
            "No {} matches {:?}.",
            subject.thing(),
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match showing.note {
        Some(reason) => {
            Notice::loud("Nothing here, and the reading is incomplete.").saying(format!(
                "Reason: {reason}. It may or may not be about this list; the summary screen \
                 carries the whole of it."
            ))
        }
        None => Notice::plain(format!("No {} in this reading.", subject.thing())).saying(
            match subject {
                Subject::LoggedIn => {
                    "This reading names no session and no source of logins either. An agent \
                     older than this console says nothing about its sources; a newer one \
                     always lists them, answering or not."
                }
                Subject::Keys | Subject::SshUsers => {
                    "No authorized_keys file was found for any account the agent could look at."
                }
                _ => "This reading was empty.",
            },
        ),
    }
}
