use vigil_model::{ControlReport, ControlTarget, Controlling, Request, Response};

use crate::ui::app::App;

use super::sheet::{headline, said};
use crate::ui::{Choosing, Paper};

pub const CONTROL: char = 'U';

pub const NOTHING_TO_CONTROL: &str =
    "Nothing here is started or stopped: U works on the startup screen.";

fn nothing_under_the_cursor(target: ControlTarget) -> String {
    format!(
        "There is nothing here to act on: no row is marked, and the cursor is not on a {}.",
        target.named()
    )
}

impl App {
    pub(in crate::ui::app) fn control_target(&self) -> Option<ControlTarget> {
        match self.marking_offered() {
            true => self.pane().and_then(|pane| pane.offers().controlling),
            false => None,
        }
    }

    pub(in crate::ui::app) fn controlling(&mut self) {
        let Some(target) = self.control_target() else {
            self.message = Some(NOTHING_TO_CONTROL.to_string());
            return;
        };
        if self.what_a_kill_would_take().is_empty() {
            self.message = Some(nothing_under_the_cursor(target));
            return;
        }

        self.chooser
            .open_by_key(Choosing::Control(target), ways(target));
    }

    pub(in crate::ui::app) fn chose_a_way_of_controlling(
        &mut self,
        target: ControlTarget,
        at: usize,
    ) {
        let Some(controlling) = target.ways().get(at).copied() else {
            return;
        };
        self.ask_the_agent_to_control(target, controlling);
    }

    fn ask_the_agent_to_control(&mut self, target: ControlTarget, controlling: Controlling) {
        let keys = self.what_a_kill_would_take();
        if keys.is_empty() {
            self.message = Some(nothing_under_the_cursor(target));
            return;
        }

        let answers = match self.link.ask(&[Request::control(keys, controlling)]) {
            Ok(answers) => answers,
            Err(trouble) => {
                self.message = Some(format!(
                    "{}: {}",
                    trouble.headline(),
                    trouble.what_to_try().join(" ")
                ));
                return;
            }
        };

        match answers.into_iter().next() {
            Some(Response::Controlled { report }) => self.took_the_control(&report),
            Some(Response::Error { error }) => {
                self.paper = Some(Paper::of(
                    "THE AGENT DID NOTHING",
                    vec![error.message, String::new(), refusal_advice(&error.code)],
                ));
            }
            Some(other) => {
                self.message = Some(format!(
                    "The agent answered a question nobody asked it: {other:?}"
                ));
            }
            None => {
                self.message = Some("The agent closed the connection without an answer.".into())
            }
        }
        self.refresh_wanted = true;
    }

    fn took_the_control(&mut self, report: &ControlReport) {
        if let Some(panes) = self.panes_mut() {
            for one in report.controlled.iter().filter(|one| one.done) {
                panes.mark(&one.key, false);
            }
        }
        self.paper = Some(Paper::of(headline(report), said(report)));
    }
}

fn ways(target: ControlTarget) -> Vec<(char, String)> {
    target
        .ways()
        .iter()
        .map(|controlling| (key_of(*controlling), controlling.said().to_string()))
        .collect()
}

fn key_of(controlling: Controlling) -> char {
    match controlling {
        Controlling::Stop => 'S',
        Controlling::Start => 'R',
        Controlling::Disable => 'D',
        Controlling::Enable => 'E',
        Controlling::Mask => 'M',
        Controlling::Unmask => 'U',
        Controlling::Comment => 'O',
        Controlling::Uncomment => 'B',
    }
}

fn refusal_advice(code: &str) -> String {
    match code {
        vigil_model::ProtocolError::NOT_ALLOWED => {
            "This is off until collectors/persistence.yaml says units.from_the_console: true, and the daemon \
             reads that key once, at start-up. It is a key of its own: switching killing or \
             accounts on does not switch this on."
                .to_string()
        }
        vigil_model::ProtocolError::UNKNOWN_QUERY => {
            "This agent is older than this console and starts or stops nothing: upgrade the \
             agent."
                .to_string()
        }
        _ => "The agent refused and said why above.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_way_of_controlling_is_offered_in_the_order_the_protocol_lists_them() {
        for target in ControlTarget::ALL {
            assert_eq!(ways(*target).len(), target.ways().len());
            for (at, controlling) in target.ways().iter().enumerate() {
                assert_eq!(ways(*target)[at].1, controlling.said());
            }
        }
    }

    #[test]
    fn each_way_answers_to_a_letter_of_its_own_and_none_of_them_is_the_cancel_key() {
        for target in ControlTarget::ALL {
            let mut keys: Vec<char> = ways(*target).into_iter().map(|(key, _)| key).collect();
            let offered = keys.len();
            keys.sort_unstable();
            keys.dedup();

            assert_eq!(keys.len(), offered, "two ways share a letter");
            assert!(
                !keys.contains(&crate::ui::CANCEL),
                "the key that walks away must never also be a key that stops a service"
            );
        }
    }

    #[test]
    fn the_key_that_opens_the_band_is_not_one_the_lists_of_this_console_already_answer_to() {
        for taken in [
            crate::ui::app::deeds::MARK,
            crate::ui::app::deeds::UNMARK_EVERY,
            crate::ui::app::deeds::SUPPRESS,
            crate::ui::app::deeds::KILL,
            crate::ui::app::deeds::NEW,
            crate::ui::app::deeds::EDIT,
            crate::ui::app::deeds::DELETE,
        ] {
            assert_ne!(
                CONTROL, taken,
                "two things on one key is a key that does the other one on the screen where \
                 the reader learned the first"
            );
        }
    }
}
