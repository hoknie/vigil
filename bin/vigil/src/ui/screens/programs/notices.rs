use super::showing::Showing;
use crate::ui::helpers::words::{moment, refusal};
use crate::ui::{Notice, Program, Reading, View};

pub(super) fn missing(view: &View, showing: &Showing<'_>) -> Option<Notice> {
    let program = showing.program;
    if view.switched_off(program.collector()) {
        return Some(
            Notice::plain(format!(
                "This agent is not reading {}.",
                match program {
                    Program::Running => "the programs running here",
                    Program::Launches => "what people run here",
                }
            ))
            .saying(
                view.collector_reason(program.collector())
                    .unwrap_or("switched off in the configuration")
                    .to_string(),
            )
            .saying("The console asks for no reading it was told is switched off."),
        );
    }

    match view.reading(program.collector()) {
        Reading::Unknown => Some(
            Notice::plain("The agent has not been asked yet.")
                .saying("Press r to ask now; otherwise every couple of seconds."),
        ),
        Reading::NotTakenYet => Some(
            Notice::plain("The agent has not taken this reading yet.")
                .saying("The first one is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(refusal::refused(
            refusal,
            program.collector(),
            match program {
                Program::Running => "No running program is listed here: nothing was read.",
                Program::Launches => "No launch is listed here: nothing was read.",
            },
        )),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::plain(format!(
                "The reading taken at {} holds nothing at all.",
                moment::time_of_day(&snapshot.taken_at)
            ))
            .saying(match program {
                Program::Running => {
                    "Every host runs something, this agent among them. Treat this as a \
                     reading that saw nothing."
                }
                Program::Launches => {
                    "Nothing has been run since this agent started watching. This list only \
                     grows."
                }
            }),
        ),
        Reading::Taken(_) => None,
    }
}
