use ratatui::crossterm::event::KeyCode;
use vigil_model::{AccountChange, Changing};
use vigil_view::RowKey;

use super::asked::Asked;
use super::changing::NOT_READ_YET;
use super::sentence::sentence;
use crate::ui::app::App;
use crate::ui::helpers::motion::form;
use crate::ui::{Editing, Pressed, Reading};

const NOTHING_UNDER_THE_CURSOR: &str = "There is no row under the cursor to edit.";

impl App {
    pub(super) fn open_the_form(&mut self, changing: Changing) {
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

    pub(in crate::ui::app) fn walk_the_form(&mut self, code: KeyCode) {
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

    pub(in crate::ui::app) fn change_the_form_asks_for(&self) -> Result<AccountChange, String> {
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
}
