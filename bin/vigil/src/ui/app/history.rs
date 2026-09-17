use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::App;

use crate::ui::details::pieces;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::motion::keys;
use crate::ui::{Action, History, Reading};

pub const HISTORY: char = 'H';

const NO_HISTORY_HERE: &str = "This list keeps no history of its rows: H lists the runs of a \
                               launch, on the second list of the programs section.";

const NOTHING_UNDER_THE_CURSOR: &str = "There is no row under the cursor to show the history of.";

const NOT_READ_YET: &str = "This list has not been read yet, so there is no history to show.";

impl App {
    pub(in crate::ui::app) fn open_the_history(&mut self) -> bool {
        let Some(pane) = self.pane() else {
            return false;
        };
        if !pane.offers().history {
            self.message = Some(NO_HISTORY_HERE.to_string());
            return false;
        }
        let row = match self.pane_row_under_the_cursor() {
            Some(row) if row.of_the_reading => row,
            _ => {
                self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
                return false;
            }
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            self.message = Some(NOT_READ_YET.to_string());
            return false;
        };

        let said = pane.history(snapshot, &row);
        if said.is_empty() {
            self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
            return false;
        }
        self.history = Some(History::open(said));
        self.chooser.close();
        false
    }

    pub(super) fn walk_the_history(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match keys::action(code, modifiers, false) {
            Action::Leave => self.leaving = true,
            Action::Back | Action::Sideways(-1) => {
                self.history = None;
                self.settle();
            }
            Action::Letter(crate::ui::app::clicks::MOUSE) => self.switch_the_mouse(),
            Action::Move(motion) | Action::Pick(motion) | Action::Gather(motion) => {
                let drawing = pieces::drawing(self.body.get());
                let page = drawing.height as usize;
                let width = self.look.text_width(drawing.width);
                let look = self.look;
                if let Some(history) = self.history.as_mut() {
                    let total = pieces::height(history.pieces(), Acts::default(), look, width);
                    history.scroll(motion, page, total);
                }
            }
            _ => {}
        }
    }
}
