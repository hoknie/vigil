use vigil_model::{AccountChange, AccountObject, ChangeReport, Changing, Request};

use super::answered::answered;
use super::asked::Asked;
use super::sentence::not_offered;
use super::sheet::{headline, said};
use crate::ui::app::App;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::{Paper, Reading, holding};

pub const NEW: char = 'n';

pub const EDIT: char = 'e';

pub const DELETE: char = 'D';

pub const NOTHING_TO_CHANGE: &str = "Nothing on this list is changed from this console: the \
                                     accounts of this host are, with n, e and D.";

pub(super) const NOT_READ_YET: &str = "the accounts have not been read yet, so there is nothing to fill a \
                            form from or to delete";

impl App {
    pub(in crate::ui::app) fn changes_offered(&self) -> Option<AccountObject> {
        holding(self.nav.at().name())?;
        self.pane().and_then(|pane| pane.offers().changing)
    }

    pub(in crate::ui::app) fn changing_by_key(&mut self, key: char) -> bool {
        match key {
            NEW => self.open_the_form(Changing::Create),
            EDIT => self.open_the_form(Changing::Update),
            DELETE => self.deleting(),
            _ => {}
        }
        true
    }

    pub(in crate::ui::app) fn account_acts(&self) -> Acts {
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

    pub(super) fn offered_here(&mut self, changing: Changing) -> Option<AccountObject> {
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

    pub(super) fn ask_for(&self, changes: Vec<AccountChange>) -> Asked {
        match self.link.ask(&[Request::Change { changes }]) {
            Ok(answers) => answered(answers.into_iter().next()),
            Err(trouble) => Asked::Trouble(format!(
                "{}: {}",
                trouble.headline(),
                trouble.what_to_try().join(" ")
            )),
        }
    }

    pub(super) fn took_the_changes(&mut self, report: &ChangeReport, left: &[(String, String)]) {
        if let Some(panes) = self.panes_mut() {
            for changed in report.changed.iter().filter(|changed| changed.done) {
                panes.mark(&changed.key, false);
            }
        }
        self.paper = Some(Paper::of(headline(report, left), said(report, left)));
    }
}
