use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::WATCHING;
use crate::ui::screens::{findings, home};
use crate::ui::types::focus::position::keyed::Keyed;
use crate::ui::{Aim, Level, Offset, Screen, Spot, Target};

impl App {
    pub(super) fn press_the_target(&mut self, target: Target) {
        match target {
            Target::Aim(Aim::Option(at)) => self.press_the_option(at),
            Target::Aim(Aim::Choice(at)) => self.press_the_choice(at),
            Target::Aim(Aim::Spot(spot)) => self.press_the_spot(spot),
            Target::Aim(Aim::Popup) => {}
            Target::Button(at) => self.press_the_button_drawn(at),
            Target::Back => self.pressed(KeyCode::Esc),
            Target::Counting => self.pressed(KeyCode::Char(WATCHING)),
            Target::Row(at) => self.press_the_row(at),
            Target::Pane(at) => self.press_the_name(at),
            Target::Group(at) => self.press_the_group(at),
            Target::List | Target::Detail => {}
        }
    }

    fn press_the_option(&mut self, at: usize) {
        let by = at as isize - self.chooser.at() as isize;
        self.chooser.step(by);
        self.chose();
        self.settle();
    }

    fn press_the_choice(&mut self, choice: usize) {
        let Some(editing) = self.editing.as_mut() else {
            return;
        };
        let Some(dropdown) = editing.dropdown() else {
            return;
        };
        let Some(place) = dropdown
            .shown(editing.choices())
            .iter()
            .position(|shown| *shown == choice)
        else {
            return;
        };
        editing.walk_the_list(place as isize - dropdown.at() as isize);
        self.pressed(KeyCode::Char(' '));
    }

    fn press_the_spot(&mut self, spot: Spot) {
        let Some(editing) = self.editing.as_mut() else {
            return;
        };
        let already = editing.spot() == spot;
        editing.focus(spot);
        match spot {
            Spot::Back | Spot::Save | Spot::Cancel => self.pressed(KeyCode::Enter),
            Spot::Field(_) if already && !self.in_a_text() => self.pressed(KeyCode::Char(' ')),
            Spot::Field(_) => {}
        }
    }

    fn in_a_text(&self) -> bool {
        self.editing
            .as_ref()
            .is_some_and(crate::ui::Editing::in_a_text)
    }

    fn press_the_button_drawn(&mut self, at: usize) {
        self.level = Level::Detail;
        self.in_the_buttons = true;
        self.button = at;
        self.press_the_button();
        self.settle();
    }

    fn press_the_row(&mut self, at: usize) {
        if self.on_the_silenced() {
            self.point_at_the_silenced(at);
            return;
        }
        if self.level == Level::List && self.row_the_cursor_is_on() == Some(at) {
            self.pressed(KeyCode::Right);
            return;
        }
        self.point_the_cursor_at_the_row(at);
        self.level = Level::List;
        self.settle();
    }

    fn row_the_cursor_is_on(&self) -> Option<usize> {
        match self.nav.at() {
            Screen::HOME => Some(self.nav.sections.at()),
            Screen::SUMMARY => None,
            Screen::FINDINGS => Some(self.nav.findings.at()),
            screen if screen.draws_a_reading() => self.panes().map(crate::ui::Panes::at),
            _ => None,
        }
    }

    fn point_the_cursor_at_the_row(&mut self, at: usize) {
        self.picked.let_go();
        match self.nav.at() {
            Screen::HOME => {
                let keys = home::keys(&self.view);
                if let Some(key) = keys.get(at).cloned() {
                    self.nav.sections.point_at(&key, &keys);
                }
            }
            Screen::FINDINGS => {
                let keys = findings::keys(&self.passing());
                if let Some(key) = keys.get(at).cloned() {
                    self.nav.findings.point_at(&key, &keys);
                }
                self.nav.difference = Offset::default();
            }
            screen if screen.draws_a_reading() => {
                let rows = self.pane_listed();
                let key = rows.key_at(at).map(str::to_string);
                if let Some(key) = key
                    && let Some(panes) = self.panes_mut()
                {
                    panes.cursor_mut().point_at(&key, rows.as_ref());
                }
                self.nav.difference = Offset::default();
                self.rest_the_buttons();
            }
            _ => {}
        }
    }

    fn press_the_name(&mut self, at: usize) {
        if self.nav.at() == Screen::FINDINGS {
            if self.findings_list() != at {
                self.choose_the_findings_list(at);
                self.settle();
            }
            self.level = Level::of_the_lists(self.rungs());
            return;
        }
        if self.panes().is_some_and(|panes| panes.showing() == at) {
            self.level = Level::of_the_lists(self.rungs());
            return;
        }
        if let Some(panes) = self.panes_mut() {
            panes.show(at);
        }
        self.detail_open = false;
        self.nav.difference = Offset::default();
        self.refresh_wanted = true;
        self.settle();
        self.level = Level::of_the_lists(self.rungs());
    }
}
