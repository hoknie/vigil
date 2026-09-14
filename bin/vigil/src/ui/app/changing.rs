use ratatui::crossterm::event::KeyCode;
use vigil_model::{
    AccountChange, AccountObject, ChangeReport, Changed, Changing, ProtocolError, Request, Response,
};
use vigil_view::RowKey;

use super::App;

use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::motion::form;
use crate::ui::{Choosing, Editing, Paper, Pressed, Reading, holding};

pub const NEW: char = 'n';

pub const EDIT: char = 'e';

pub const DELETE: char = 'D';

pub const NOTHING_TO_CHANGE: &str = "Nothing on this list is changed from this console: the \
                                     accounts of this host are, with n, e and D.";

const NOT_READ_YET: &str = "the accounts have not been read yet, so there is nothing to fill a \
                            form from or to delete";

const NOTHING_UNDER_THE_CURSOR: &str = "There is no row under the cursor to edit.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Asked {
    Report(ChangeReport),
    Refused {
        message: String,
        advice: Option<String>,
    },
    Trouble(String),
}

impl App {
    pub(super) fn changes_offered(&self) -> Option<AccountObject> {
        holding(self.nav.at().name())?;
        self.pane().and_then(|pane| pane.offers().changing)
    }

    pub(super) fn changing_by_key(&mut self, key: char) -> bool {
        match key {
            NEW => self.open_the_form(Changing::Create),
            EDIT => self.open_the_form(Changing::Update),
            DELETE => self.deleting(),
            _ => {}
        }
        true
    }

    pub(super) fn account_acts(&self) -> Acts {
        let (Some(object), Some(pane), Some(row)) = (
            self.changes_offered(),
            self.pane(),
            self.pane_row_under_the_cursor(),
        ) else {
            return Acts::default();
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Acts::default();
        };

        Acts::of_an_account(
            object.offers(Changing::Update)
                && pane.form(snapshot, Some(&row), Changing::Update).is_ok(),
            object.offers(Changing::Delete)
                && pane
                    .change(snapshot, Some(&row), Changing::Delete, None)
                    .is_ok(),
        )
    }

    fn offered_here(&mut self, changing: Changing) -> Option<AccountObject> {
        let Some(object) = self.changes_offered() else {
            self.message = Some(NOTHING_TO_CHANGE.to_string());
            return None;
        };
        if !object.offers(changing) {
            self.message = Some(not_offered(object, changing));
            return None;
        }
        Some(object)
    }

    fn open_the_form(&mut self, changing: Changing) {
        if self.offered_here(changing).is_none() {
            return;
        }
        let (Some(pane), Some(at)) = (self.pane(), self.panes().map(|panes| panes.showing()))
        else {
            return;
        };
        let row = match changing {
            Changing::Create => None,
            _ => match self.pane_row_under_the_cursor() {
                Some(row) if row.of_the_reading => Some(row),
                _ => {
                    self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
                    return;
                }
            },
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            self.message = Some(sentence(NOT_READ_YET));
            return;
        };

        match pane.form(snapshot, row.as_ref(), changing) {
            Ok(form) => {
                self.editing = Some(Editing::open(
                    form,
                    self.nav.at(),
                    at,
                    row.map(|row| row.key),
                    changing,
                ));
                self.chooser.close();
                self.rest_the_buttons();
            }
            Err(why) => self.message = Some(sentence(&why)),
        }
    }

    fn deleting(&mut self) {
        let Some(object) = self.offered_here(Changing::Delete) else {
            return;
        };
        let keys = self.what_a_kill_would_take();
        if keys.is_empty() {
            self.message = Some(format!(
                "There is nothing here to delete: no row is marked, and the cursor is not on a {}.",
                object.named()
            ));
            return;
        }
        let (changes, left) = self.deletions(&keys);
        if changes.is_empty() {
            self.message = left
                .first()
                .map(|(key, why)| sentence(&format!("{key}: {why}")));
            return;
        }

        self.chooser.open_by_key(
            Choosing::Delete(object),
            vec![(DELETE, object.deleting().to_string())],
        );
    }

    pub(super) fn deletions(&self, keys: &[String]) -> (Vec<AccountChange>, Vec<(String, String)>) {
        let Some(pane) = self.pane() else {
            return (Vec::new(), Vec::new());
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return (
                Vec::new(),
                keys.iter()
                    .map(|key| (key.clone(), NOT_READ_YET.to_string()))
                    .collect(),
            );
        };

        let mut changes = Vec::new();
        let mut left = Vec::new();
        for key in keys {
            if !snapshot.items.contains_key(key) {
                left.push((
                    key.clone(),
                    "no longer in the reading: it changed or went away since it was marked"
                        .to_string(),
                ));
                continue;
            }
            match pane.change(
                snapshot,
                Some(&RowKey::of(key.clone())),
                Changing::Delete,
                None,
            ) {
                Ok(change) => changes.push(change),
                Err(why) => left.push((key.clone(), why)),
            }
        }
        (changes, left)
    }

    pub(super) fn chose_to_delete(&mut self) {
        let keys = self.what_a_kill_would_take();
        let (changes, left) = self.deletions(&keys);
        if changes.is_empty() {
            self.message = Some(
                "Nothing was asked for: every row that was to be deleted is gone from the \
                 reading or cannot be deleted."
                    .to_string(),
            );
            return;
        }

        match self.ask_for(changes) {
            Asked::Report(report) => self.took_the_changes(&report, &left),
            Asked::Refused { message, advice } => {
                let mut lines = vec![message];
                if let Some(advice) = advice {
                    lines.push(String::new());
                    lines.push(advice);
                }
                self.paper = Some(Paper::of("THE AGENT DID NOTHING", lines));
            }
            Asked::Trouble(said) => self.message = Some(said),
        }
        self.refresh_wanted = true;
    }

    pub(super) fn walk_the_form(&mut self, code: KeyCode) {
        let Some(editing) = self.editing.as_mut() else {
            return;
        };
        match form::pressed(editing, code) {
            Pressed::Nothing => {}
            Pressed::Leave => {
                self.editing = None;
                self.settle();
            }
            Pressed::Submit => self.submit_the_form(),
        }
    }

    pub(super) fn change_the_form_asks_for(&self) -> Result<AccountChange, String> {
        let Some(editing) = &self.editing else {
            return Err("no form is open".to_string());
        };
        let Some(section) = self.section_of(editing.screen()) else {
            return Err("the section this form was opened on is no longer drawn".to_string());
        };
        let panes = section.panes();
        let Some(pane) = panes.get(editing.pane()) else {
            return Err("the list this form was opened on is no longer drawn".to_string());
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Err(NOT_READ_YET.to_string());
        };
        if let Some(key) = editing.row()
            && !snapshot.items.contains_key(key)
        {
            return Err(format!(
                "{key} is no longer in the reading: it changed or went away since this form was \
                 opened, and nothing was sent"
            ));
        }

        let row = editing.row().map(RowKey::of);
        let change = pane.change(
            snapshot,
            row.as_ref(),
            editing.changing(),
            Some(editing.form()),
        )?;
        if change.is_empty() {
            return Err(
                "nothing was changed: every field is as the reading has it, so nothing was sent"
                    .to_string(),
            );
        }
        Ok(change)
    }

    fn submit_the_form(&mut self) {
        let change = match self.change_the_form_asks_for() {
            Ok(change) => change,
            Err(why) => {
                self.say_on_the_form(sentence(&why));
                return;
            }
        };

        match self.ask_for(vec![change]) {
            Asked::Report(report) if !report.changed.is_empty() && report.refused() == 0 => {
                self.editing = None;
                self.took_the_changes(&report, &[]);
            }
            Asked::Report(report) => {
                let refused: Vec<String> = report
                    .changed
                    .iter()
                    .filter(|one| !one.done)
                    .map(|one| sentence(&one.said))
                    .collect();
                self.say_on_the_form(match refused.is_empty() {
                    true => "The agent answered with nothing done and nothing refused.".to_string(),
                    false => format!("The agent did not do it. {}", refused.join(" ")),
                });
            }
            Asked::Refused { message, advice } => self.say_on_the_form(match advice {
                Some(advice) => format!("{} {advice}", sentence(&message)),
                None => sentence(&message),
            }),
            Asked::Trouble(said) => self.say_on_the_form(said),
        }
        self.refresh_wanted = true;
    }

    fn say_on_the_form(&mut self, said: String) {
        if let Some(editing) = self.editing.as_mut() {
            editing.said(said);
        }
    }

    fn ask_for(&self, changes: Vec<AccountChange>) -> Asked {
        match self.link.ask(&[Request::Change { changes }]) {
            Ok(answers) => answered(answers.into_iter().next()),
            Err(trouble) => Asked::Trouble(format!(
                "{}: {}",
                trouble.headline(),
                trouble.what_to_try().join(" ")
            )),
        }
    }

    fn took_the_changes(&mut self, report: &ChangeReport, left: &[(String, String)]) {
        if let Some(panes) = self.panes_mut() {
            for changed in report.changed.iter().filter(|changed| changed.done) {
                panes.mark(&changed.key, false);
            }
        }
        self.paper = Some(Paper::of(headline(report, left), said(report, left)));
    }
}

pub(super) fn answered(answer: Option<Response>) -> Asked {
    match answer {
        Some(Response::Changed { report }) => Asked::Report(*report),
        Some(Response::Error { error }) => Asked::Refused {
            advice: advice(&error.code),
            message: error.message,
        },
        Some(other) => Asked::Trouble(format!(
            "The agent answered a question nobody asked it: {other:?}"
        )),
        None => Asked::Trouble("The agent closed the connection without an answer.".to_string()),
    }
}

fn advice(code: &str) -> Option<String> {
    match code {
        ProtocolError::NOT_ALLOWED => Some(
            "Changing accounts from the console is off until vigil.yaml says \
             accounts.from_the_console: true, and the daemon reads that key once, at start-up."
                .to_string(),
        ),
        ProtocolError::UNKNOWN_QUERY => Some(
            "This agent is older than this console and changes no accounts: upgrade the agent."
                .to_string(),
        ),
        _ => None,
    }
}

fn not_offered(object: AccountObject, changing: Changing) -> String {
    let done = match changing {
        Changing::Create => "created",
        Changing::Update => "edited",
        Changing::Delete => "deleted",
    };
    sentence(&format!(
        "{}s are not {done} from this console",
        object.named()
    ))
}

fn sentence(said: &str) -> String {
    let said = said.trim();
    let mut characters = said.chars();
    let mut out: String = match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => return String::new(),
    };
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}

fn headline(report: &ChangeReport, left: &[(String, String)]) -> String {
    format!(
        "{} OF {} CHANGE(S)",
        report.done(),
        report.changed.len() + left.len()
    )
}

fn said(report: &ChangeReport, left: &[(String, String)]) -> Vec<String> {
    let mut lines = vec![format!("at {}", report.acted_at), String::new()];
    for changed in &report.changed {
        lines.push(one(changed));
    }
    for (key, why) in left {
        lines.push(format!(
            "  [left] {key} — {why} (the console did not ask for it)"
        ));
    }
    lines.push(String::new());
    lines.push(match report.refused() + left.len() {
        0 => "The agent reads the accounts of this host again straight after a change: the \
              list shows this within a few seconds."
            .to_string(),
        refused => format!(
            "{refused} of them were not done, for the reason written beside each. The agent \
             reads the accounts again straight away, and the list shows the rest within a few \
             seconds."
        ),
    });
    lines
}

fn one(changed: &Changed) -> String {
    let mark = match changed.done {
        true => "done",
        false => "left",
    };
    format!("  [{mark}] {} — {}", changed.key, changed.said)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deleted(name: &str) -> AccountChange {
        AccountChange::DeleteUser { name: name.into() }
    }

    fn report(changed: Vec<Changed>) -> ChangeReport {
        ChangeReport {
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            changed,
        }
    }

    #[test]
    fn the_sheet_counts_what_was_done_names_what_was_left_and_says_when_the_list_catches_up() {
        let report = report(vec![
            Changed::done(&deleted("contractor"), "userdel finished"),
            Changed::refused(&deleted("root"), "uid 0 is not deleted"),
        ]);
        let left = vec![(
            "session-source|logind".to_string(),
            "a record of where logins are read from is not a session".to_string(),
        )];

        assert_eq!(headline(&report, &left), "1 OF 3 CHANGE(S)");
        let page = said(&report, &left).join("\n");
        assert!(
            page.contains("[done] account|contractor — userdel finished"),
            "{page}"
        );
        assert!(
            page.contains("[left] account|root — uid 0 is not deleted"),
            "{page}"
        );
        assert!(page.contains("session-source|logind"), "{page}");
        assert!(page.contains("2 of them were not done"), "{page}");
    }

    #[test]
    fn a_sheet_where_everything_was_done_says_the_accounts_are_read_again_straight_away() {
        let report = report(vec![Changed::done(&deleted("contractor"), "done")]);

        let page = said(&report, &[]).join("\n");

        assert!(page.contains("again straight after a change"), "{page}");
        assert!(
            !page.contains("every few minutes"),
            "the list no longer waits for the period of the accounts, and a sheet that says it \
             does sends a person away for five minutes for nothing: {page}"
        );
    }

    #[test]
    fn an_agent_that_has_it_switched_off_is_answered_with_the_key_that_switches_it_on() {
        let asked = answered(Some(Response::Error {
            error: ProtocolError::new(ProtocolError::NOT_ALLOWED, "changing accounts is off"),
        }));

        match asked {
            Asked::Refused { message, advice } => {
                assert_eq!(message, "changing accounts is off");
                assert!(
                    advice
                        .expect("advice")
                        .contains("accounts.from_the_console"),
                    "a refusal that does not name the key sends a reader looking through the file"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_agent_older_than_the_verb_is_said_to_be_older_and_not_broken() {
        match answered(Some(Response::Error {
            error: ProtocolError::new(ProtocolError::UNKNOWN_QUERY, "unknown query \"change\""),
        })) {
            Asked::Refused { advice, .. } => {
                assert!(advice.expect("advice").contains("older"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_connection_closed_without_an_answer_is_trouble_in_words() {
        assert!(
            matches!(answered(None), Asked::Trouble(said) if said.contains("without an answer"))
        );
    }

    #[test]
    fn a_change_a_list_does_not_offer_is_refused_in_a_sentence_naming_the_object() {
        assert_eq!(
            not_offered(AccountObject::User, Changing::Create),
            "Accounts are not created from this console."
        );
    }

    #[test]
    fn what_the_daemon_or_a_pane_says_is_turned_into_a_sentence_once() {
        assert_eq!(sentence("the row is gone"), "The row is gone.");
        assert_eq!(sentence("Already said."), "Already said.");
        assert_eq!(sentence(""), "");
    }
}
