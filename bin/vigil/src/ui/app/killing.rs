use vigil_model::{KillReport, Killed, Killing, Request, Response};

use super::App;

use crate::ui::{Choosing, Level, Paper};

pub const KILL: char = 'K';

const NOTHING_TO_CLOSE: &str = "This list is read, not acted on. The sockets of this host are \
                                the one thing this console asks the agent to close.";

const NOTHING_UNDER_THE_CURSOR: &str = "There is nothing here to close: no row is marked, and \
                                        the cursor is not on a socket.";

impl App {
    pub(super) fn killing(&mut self) {
        if !self.marking_offered() {
            self.message = Some(NOTHING_TO_CLOSE.to_string());
            return;
        }
        if self.what_a_kill_would_take().is_empty() {
            self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
            return;
        }

        self.chooser.open_by_key(Choosing::Kill, ways());
    }

    pub(super) fn what_a_kill_would_take(&self) -> Vec<String> {
        let this_one = match self.pane_row_under_the_cursor() {
            Some(row) if row.of_the_reading => vec![row.key],
            _ => Vec::new(),
        };
        if self.level == Level::Detail {
            return this_one;
        }

        match self.marked().is_empty() {
            true => this_one,
            false => self.marked(),
        }
    }

    pub(super) fn chose_a_way_of_killing(&mut self, at: usize) {
        let Some(killing) = Killing::ALL.get(at).copied() else {
            return;
        };
        self.ask_the_agent_to_kill(killing);
    }

    fn ask_the_agent_to_kill(&mut self, killing: Killing) {
        let sockets = self.what_a_kill_would_take();
        if sockets.is_empty() {
            self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
            return;
        }

        let answers = match self.link.ask(&[Request::Kill {
            sockets: sockets.clone(),
            killing,
        }]) {
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
            Some(Response::Killed { report }) => self.took_it(&report),
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

    fn took_it(&mut self, report: &KillReport) {
        if let Some(panes) = self.panes_mut() {
            for killed in report.killed.iter().filter(|killed| killed.done) {
                panes.mark(&killed.key, false);
            }
        }
        self.paper = Some(Paper::of(
            format!("{} OF {} SOCKET(S)", report.done(), report.killed.len()),
            said(report),
        ));
    }
}

fn ways() -> Vec<(char, String)> {
    Killing::ALL
        .iter()
        .map(|killing| (key_of(*killing), killing.said().to_string()))
        .collect()
}

fn key_of(killing: Killing) -> char {
    match killing {
        Killing::Terminate => 'S',
        Killing::Kill => 'K',
        Killing::Destroy => 'D',
    }
}

fn refusal_advice(code: &str) -> String {
    match code {
        vigil_model::ProtocolError::NOT_ALLOWED => {
            "This is off until the configuration says otherwise. It is a key in vigil.yaml, \
             beside suppressions, and the daemon reads it at start-up."
                .to_string()
        }
        _ => "The agent refused and said why above.".to_string(),
    }
}

fn said(report: &KillReport) -> Vec<String> {
    let mut lines = vec![
        format!("{} at {}", report.killing.said(), report.acted_at),
        String::new(),
    ];
    for killed in &report.killed {
        lines.push(one(killed));
    }
    lines.push(String::new());
    lines.push(match report.refused() {
        0 => "The next reading says whether the port is closed. Nothing here is a promise \
              that it is."
            .to_string(),
        _ => format!(
            "{} of them the agent did not touch, for the reason written beside each.",
            report.refused()
        ),
    });
    lines
}

fn one(killed: &Killed) -> String {
    let mark = match killed.done {
        true => "done",
        false => "left",
    };
    let who = match (&killed.program, killed.pid) {
        (Some(program), Some(pid)) => format!("{program} (pid {pid})"),
        (None, Some(pid)) => format!("pid {pid}"),
        _ => String::new(),
    };
    match who.is_empty() {
        true => format!("  [{mark}] {} — {}", killed.key, killed.said),
        false => format!("  [{mark}] {} — {who} — {}", killed.key, killed.said),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(killing: Killing, killed: Vec<Killed>) -> KillReport {
        KillReport {
            killing,
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            killed,
        }
    }

    #[test]
    fn the_sheet_names_every_socket_the_agent_left_alone_and_why() {
        let lines = said(&report(
            Killing::Destroy,
            vec![
                Killed::done(
                    "tcp|0.0.0.0:4444",
                    30211,
                    Some("/tmp/.x/nc".into()),
                    "closed",
                ),
                Killed::refused("unix|/run/app.sock", "a unix socket is not closed this way"),
            ],
        ));
        let page = lines.join("\n");

        assert!(page.contains("unix|/run/app.sock"), "{page}");
        assert!(
            page.contains("a unix socket is not closed this way"),
            "{page}"
        );
        assert!(page.contains("pid 30211"), "{page}");
        assert!(
            page.contains("1 of them the agent did not touch"),
            "a count of what worked with no count of what did not is the half of this sheet \
             an operator would have to work out for themselves: {page}"
        );
    }

    #[test]
    fn every_way_of_killing_is_offered_in_the_order_the_protocol_lists_them() {
        assert_eq!(ways().len(), Killing::ALL.len());
        for (at, killing) in Killing::ALL.iter().enumerate() {
            assert_eq!(ways()[at].1, killing.said());
        }
    }

    #[test]
    fn each_way_answers_to_a_letter_of_its_own_and_none_of_them_is_the_cancel_key() {
        let mut keys: Vec<char> = ways().into_iter().map(|(key, _)| key).collect();
        let offered = keys.len();
        keys.sort_unstable();
        keys.dedup();

        assert_eq!(keys.len(), offered, "two ways share a letter");
        assert!(
            !keys.contains(&crate::ui::CANCEL),
            "the key that walks away must never also be a key that kills"
        );
    }
}
