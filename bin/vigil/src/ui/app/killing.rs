use vigil_model::{KillReport, KillTarget, Killed, Killing, Request, Response};

use super::App;

use crate::ui::{Choosing, Level, Paper};

pub const KILL: char = 'K';

const NOTHING_TO_ACT_ON: &str = "This list is read, not acted on. The sockets and the running \
                                 programs of this host are what this console asks the agent \
                                 to close or to stop.";

fn nothing_under_the_cursor(target: KillTarget) -> String {
    let (verb, noun) = match target {
        KillTarget::Socket => ("close", "socket"),
        KillTarget::Program => ("stop", "program"),
    };
    format!("There is nothing here to {verb}: no row is marked, and the cursor is not on a {noun}.")
}

impl App {
    pub(super) fn kill_target(&self) -> Option<KillTarget> {
        match self.marking_offered() {
            true => self.pane().and_then(|pane| pane.offers().killing),
            false => None,
        }
    }

    pub(super) fn killing(&mut self) {
        let Some(target) = self.kill_target() else {
            self.message = Some(NOTHING_TO_ACT_ON.to_string());
            return;
        };
        if self.what_a_kill_would_take().is_empty() {
            self.message = Some(nothing_under_the_cursor(target));
            return;
        }

        self.chooser
            .open_by_key(Choosing::Kill(target), ways(target));
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

    pub(super) fn chose_a_way_of_killing(&mut self, target: KillTarget, at: usize) {
        let Some(killing) = target.ways().get(at).copied() else {
            return;
        };
        self.ask_the_agent_to_kill(target, killing);
    }

    fn ask_the_agent_to_kill(&mut self, target: KillTarget, killing: Killing) {
        let keys = self.what_a_kill_would_take();
        if keys.is_empty() {
            self.message = Some(nothing_under_the_cursor(target));
            return;
        }

        let answers = match self.link.ask(&[Request::kill(target, keys, killing)]) {
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
        self.paper = Some(Paper::of(headline(report), said(report)));
    }
}

fn ways(target: KillTarget) -> Vec<(char, String)> {
    target
        .ways()
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

fn headline(report: &KillReport) -> String {
    format!(
        "{} OF {} {}(S)",
        report.done(),
        report.killed.len(),
        report.target.as_str().to_uppercase()
    )
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
    lines.push(match (report.refused(), report.target) {
        (0, KillTarget::Socket) => "The next reading says whether the port is closed. Nothing \
                                    here is a promise that it is."
            .to_string(),
        (0, KillTarget::Program) => "The next reading says whether the program still runs. A \
                                     process asked to stop may take its time over it, or not \
                                     stop at all."
            .to_string(),
        (refused, _) => format!(
            "{refused} of them the agent did not touch, for the reason written beside each."
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
        (Some(program), None) => program.clone(),
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
        report_about(KillTarget::Socket, killing, killed)
    }

    fn report_about(target: KillTarget, killing: Killing, killed: Vec<Killed>) -> KillReport {
        KillReport {
            target,
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
    fn a_sheet_about_programs_counts_programs_and_says_what_the_next_reading_will_tell() {
        let report = report_about(
            KillTarget::Program,
            Killing::Terminate,
            vec![Killed::done(
                "exec|/tmp/.x/nc|www-data",
                9001,
                Some("/tmp/.x/nc".into()),
                "SIGTERM sent to 2 process(es): 9001, 9002",
            )],
        );

        assert_eq!(headline(&report), "1 OF 1 PROGRAM(S)");
        let page = said(&report).join("\n");
        assert!(page.contains("whether the program still runs"), "{page}");
        assert!(!page.contains("port"), "{page}");
    }

    #[test]
    fn every_way_of_killing_is_offered_in_the_order_the_protocol_lists_them() {
        for target in [KillTarget::Socket, KillTarget::Program] {
            assert_eq!(ways(target).len(), target.ways().len());
            for (at, killing) in target.ways().iter().enumerate() {
                assert_eq!(ways(target)[at].1, killing.said());
            }
        }
    }

    #[test]
    fn a_program_is_not_offered_the_letter_that_closes_a_socket() {
        let letters: Vec<char> = ways(KillTarget::Program)
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        assert_eq!(letters, vec!['S', 'K']);
    }

    #[test]
    fn each_way_answers_to_a_letter_of_its_own_and_none_of_them_is_the_cancel_key() {
        for target in [KillTarget::Socket, KillTarget::Program] {
            let mut keys: Vec<char> = ways(target).into_iter().map(|(key, _)| key).collect();
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
}
