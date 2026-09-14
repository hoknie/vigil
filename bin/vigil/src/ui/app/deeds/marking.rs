use vigil_view::{Piece, RowKey};

use crate::ui::app::App;

use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::finding::suppression;
use crate::ui::{Paper, Reading, holding};

pub const MARK: char = 'x';

pub const UNMARK_EVERY: char = 'M';

pub const SUPPRESS: char = 'S';

const NOTHING_TO_MARK: &str = "Nothing on this list is marked with a key: marking is offered where the console can act \
     on a row, which is the sockets, the running programs and the accounts of this host.";

const NOTHING_IS_MARKED: &str = "Nothing is marked. Press x on a row; on a program of the ports \
                                 screen it takes every socket under it.";

impl App {
    pub(in crate::ui::app) fn acts(&self) -> Acts {
        let on_a_row = self
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.of_the_reading);

        match self.kill_target() {
            Some(target) => Acts::of_a_row(on_a_row.then_some(target)),
            None if on_a_row => self.account_acts(),
            None => Acts::default(),
        }
    }

    pub(in crate::ui::app) fn buttons_here(&self) -> usize {
        match self.level == crate::ui::Level::Detail {
            true => self.acts().buttons().len(),
            false => 0,
        }
    }

    pub(in crate::ui::app) fn button_at(&self) -> Option<usize> {
        match self.in_the_buttons && self.buttons_here() > 0 {
            true => Some(self.button.min(self.buttons_here() - 1)),
            false => None,
        }
    }

    pub(in crate::ui::app) fn deeper_into_the_buttons(&mut self) -> bool {
        let buttons = self.buttons_here();
        if buttons == 0 {
            return false;
        }
        if !self.in_the_buttons {
            self.in_the_buttons = true;
            self.button = 0;
            return true;
        }
        if self.button + 1 < buttons {
            self.button += 1;
        }
        true
    }

    pub(in crate::ui::app) fn back_out_of_the_buttons(&mut self) -> bool {
        if !self.in_the_buttons || self.buttons_here() == 0 {
            return false;
        }
        match self.button {
            0 => self.in_the_buttons = false,
            _ => self.button -= 1,
        }
        true
    }

    pub(in crate::ui::app) fn rest_the_buttons(&mut self) {
        self.button = 0;
        self.in_the_buttons = false;
    }

    pub(in crate::ui::app) fn press_the_button(&mut self) -> bool {
        let Some(at) = self.button_at() else {
            return false;
        };
        let Some(button) = self.acts().button(at) else {
            return false;
        };
        match button.key {
            SUPPRESS => {
                self.show_the_suppressions();
            }
            key if key == super::accounts::EDIT || key == super::accounts::DELETE => {
                self.changing_by_key(key);
            }
            _ => self.killing(),
        }
        true
    }

    pub(in crate::ui::app) fn marking_offered(&self) -> bool {
        holding(self.nav.at().name()).is_some()
            && self.pane().is_some_and(|pane| pane.offers().marking)
    }

    pub(in crate::ui::app) fn marked(&self) -> Vec<String> {
        self.panes().map(|panes| panes.marked()).unwrap_or_default()
    }

    pub(in crate::ui::app) fn mark_under_the_cursor(&mut self) -> bool {
        if !self.marking_offered() {
            self.message = Some(NOTHING_TO_MARK.to_string());
            return false;
        }
        let Some(row) = self.pane_row_under_the_cursor() else {
            return false;
        };
        let keys = self.marked_by(&row);
        if keys.is_empty() {
            self.message = Some(
                "This row is a heading of nothing the agent read: there is nothing under it \
                 to mark."
                    .to_string(),
            );
            return false;
        }

        let wanted = !keys
            .iter()
            .all(|key| self.panes().is_some_and(|panes| panes.is_marked(key)));
        if let Some(panes) = self.panes_mut() {
            for key in &keys {
                panes.mark(key, wanted);
            }
        }
        true
    }

    fn marked_by(&self, row: &RowKey) -> Vec<String> {
        if row.of_the_reading {
            return vec![row.key.clone()];
        }

        let under = self.rows_gathered_under(&row.key);
        match under.is_empty() {
            true => self.rows_a_closed_heading_stands_for(&row.key),
            false => under,
        }
    }

    fn rows_gathered_under(&self, heading: &str) -> Vec<String> {
        self.pane_rows()
            .into_iter()
            .filter(|row| row.of_the_reading && row.gathered_under.as_deref() == Some(heading))
            .map(|row| row.key)
            .collect()
    }

    fn rows_a_closed_heading_stands_for(&self, heading: &str) -> Vec<String> {
        let (Some(pane), Some(showing)) = (self.pane(), self.showing_pane()) else {
            return Vec::new();
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Vec::new();
        };
        let hidden = showing.hidden();
        let mut opened = showing.opened();
        opened.push(heading);
        let asked = crate::ui::screens::pane::asked(&showing, &hidden, &opened);

        pane.rows(snapshot, &asked)
            .into_iter()
            .filter(|row| row.of_the_reading && row.gathered_under.as_deref() == Some(heading))
            .map(|row| row.key)
            .collect()
    }

    pub(in crate::ui::app) fn unmark_everything(&mut self) -> bool {
        if !self.marking_offered() {
            self.message = Some(NOTHING_TO_MARK.to_string());
            return false;
        }
        if let Some(panes) = self.panes_mut() {
            panes.unmark_everything();
        }
        true
    }

    pub(in crate::ui::app) fn show_the_suppressions(&mut self) -> bool {
        if !self.marking_offered() {
            self.message = Some(NOTHING_TO_MARK.to_string());
            return false;
        }
        let marked = self.what_to_suppress();
        if marked.is_empty() {
            self.message = Some(NOTHING_IS_MARKED.to_string());
            return false;
        }

        let keys: Vec<String> = marked
            .iter()
            .filter_map(|key| self.finding_key_of(key))
            .collect();
        if keys.is_empty() {
            self.message = Some(
                "The list this console is showing writes no finding key against its rows, so \
                 there is nothing to put in suppressions."
                    .to_string(),
            );
            return false;
        }

        self.paper = Some(
            Paper::of(
                format!("SUPPRESSIONS FOR {} MARKED ROW(S)", keys.len()),
                suppression::entries(&keys),
            )
            .for_copying(),
        );
        true
    }

    fn what_to_suppress(&self) -> Vec<String> {
        let this_one = match self.pane_row_under_the_cursor() {
            Some(row) if row.of_the_reading => vec![row.key],
            _ => Vec::new(),
        };
        if self.level == crate::ui::Level::Detail {
            return this_one;
        }

        match self.marked().is_empty() {
            true => this_one,
            false => self.marked(),
        }
    }

    pub(in crate::ui::app) fn finding_key_of(&self, key: &str) -> Option<String> {
        let pane = self.pane()?;
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return None;
        };

        pane.detail(snapshot, &RowKey::of(key), 80)
            .into_iter()
            .find_map(|piece| match piece {
                Piece::Key(key) => Some(key),
                _ => None,
            })
    }
}
