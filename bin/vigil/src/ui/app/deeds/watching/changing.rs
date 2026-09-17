use vigil_model::Changing;

use super::super::accounts::{DELETE, EDIT, NEW, sentence};
use crate::ui::app::App;
use crate::ui::{Editing, Reading, holding};

pub(super) const NOT_READ_YET: &str = "the watched paths have not been read yet, so there is \
                                       nothing to fill a form from or to stop watching";

const WRITTEN: &[Changing] = &[Changing::Create, Changing::Update, Changing::Delete];

impl App {
    pub(in crate::ui::app) fn written_to_the_configuration(&self) -> &'static [Changing] {
        match self.watching_offered() {
            true => WRITTEN,
            false => &[],
        }
    }

    pub(in crate::ui::app) fn watching_offered(&self) -> bool {
        holding(self.nav.at().name()).is_some()
            && self.pane().is_some_and(|pane| pane.offers().watching)
    }

    pub(in crate::ui::app) fn watching_by_key(&mut self, key: char) -> bool {
        match key {
            NEW => self.open_the_watching_form(Changing::Create),
            EDIT => self.open_the_watching_form(Changing::Update),
            DELETE => self.ask_to_stop_watching(),
            _ => return false,
        }
        true
    }

    pub(in crate::ui::app) fn a_watching_form_is_open(&self) -> bool {
        let Some(editing) = &self.editing else {
            return false;
        };
        let Some(section) = self.section_of(editing.screen()) else {
            return false;
        };

        section
            .panes()
            .get(editing.pane())
            .is_some_and(|pane| pane.offers().watching)
    }

    fn open_the_watching_form(&mut self, changing: Changing) {
        let (Some(pane), Some(at)) = (self.pane(), self.panes().map(|panes| panes.showing()))
        else {
            return;
        };
        let row = match changing {
            Changing::Create => None,
            _ => match self.pane_row_under_the_cursor() {
                Some(row) if row.of_the_reading => Some(row),
                _ => {
                    self.message =
                        Some("There is no watched path under the cursor to change.".to_string());
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
}
