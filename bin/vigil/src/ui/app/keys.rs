use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::App;

use super::clicks::MOUSE;
use crate::ui::helpers::motion::keys;
use crate::ui::screens::home;
use crate::ui::{Action, Level, Motion, Screen};

impl App {
    pub fn on_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if self.editing.is_some() {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.leaving = true;
                return;
            }
            self.message = None;
            self.walk_the_form(code, modifiers);
            return;
        }
        if self.graph.is_some() {
            self.message = None;
            self.walk_the_graph(code, modifiers);
            return;
        }
        if self.history.is_some() {
            self.message = None;
            self.walk_the_history(code, modifiers);
            return;
        }

        let action = keys::action(code, modifiers, self.typing());

        if self.asking().is_some() {
            self.message = None;
            self.walk_the_reason(action);
            self.settle();
            return;
        }

        if self.choosing() {
            self.message = None;
            if let KeyCode::Char(letter) = code
                && !modifiers.contains(KeyModifiers::CONTROL)
                && self.pressed_in_the_band(letter)
            {
                self.settle();
                return;
            }
            self.walk_the_choice(action);
            self.settle();
            return;
        }

        if self.paper.is_some() {
            self.paper = None;
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.leaving = true;
            }
            return;
        }

        if self.helping {
            self.helping = false;
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.leaving = true;
            }
            return;
        }

        if action == Action::Letter(MOUSE) {
            self.switch_the_mouse();
            return;
        }

        self.message = None;
        self.act(action);
        self.settle();
    }

    pub(super) fn act(&mut self, action: Action) {
        match action {
            Action::Leave => self.leaving = true,
            Action::Back => match self.picked_here() {
                true => self.picked.clear(),
                false => self.go_back(),
            },
            Action::Refresh => {
                self.read_the_silences_again();
                self.refresh_wanted = true;
            }
            Action::Go(screen) => self.visit(screen),
            Action::Sideways(by) => self.sideways(by),
            Action::Move(motion) => {
                self.picked.let_go();
                self.move_within(motion)
            }
            Action::Pick(motion) => self.pick_along(motion),
            Action::Gather(motion) => self.gather_along(motion),
            Action::Open => self.open(),
            Action::ToObject => self.jump_to_object(),
            Action::Sort => self.sorting(),
            Action::Kill => self.killing(),
            Action::Narrow => self.narrowing(),
            Action::Search => self.searching(),
            Action::Letter(key) => self.letter(key),
            Action::Type(character) => {
                if let Some(search) = self.search_mut() {
                    search.type_character(character);
                }
            }
            Action::Erase => {
                if let Some(search) = self.search_mut() {
                    search.erase();
                }
            }
            Action::Accept => {
                if let Some(search) = self.search_mut() {
                    search.accept();
                }
            }
            Action::Abandon => {
                if let Some(search) = self.search_mut() {
                    search.abandon();
                }
            }
            Action::Help => self.helping = true,
            Action::Ignore => {}
        }
    }

    pub(super) fn choosing(&self) -> bool {
        self.chooser.choosing().is_some()
    }

    fn pressed_in_the_band(&mut self, letter: char) -> bool {
        if self.chooser.keys().is_empty() {
            return false;
        }
        if letter == crate::ui::CANCEL {
            self.chooser.close();
            return true;
        }
        let Some(at) = self.chooser.pressed(letter) else {
            return false;
        };

        self.chooser.step(at as isize - self.chooser.at() as isize);
        self.chose();
        true
    }

    fn walk_the_reason(&mut self, action: Action) {
        match action {
            Action::Leave => self.leaving = true,
            Action::Type(character) => self.typed_into_the_reason(character),
            Action::Erase => self.erased_from_the_reason(),
            Action::Accept => self.do_the_deed(),
            Action::Abandon | Action::Back => self.left_the_reason(),
            _ => {}
        }
    }

    fn walk_the_choice(&mut self, action: Action) {
        match action {
            Action::Leave => self.leaving = true,
            Action::Move(motion) => self.chooser.step(match motion {
                Motion::Up | Motion::PageUp | Motion::First => -1,
                _ => 1,
            }),
            Action::Sideways(by) => self.chooser.step(by),
            Action::Open | Action::Accept => self.chose(),
            Action::Back | Action::Abandon => {
                self.chooser.close();
            }
            _ => {}
        }
    }

    fn searching(&mut self) {
        if self.on_the_silenced() {
            self.message = Some(super::silences::IN_THE_ORDER_OF_THE_FILES.to_string());
            return;
        }
        if self.nav.at() == Screen::HOME {
            self.message = Some(home::SEARCH_LIVES_IN_A_LIST.to_string());
            return;
        }
        match self.search_mut() {
            Some(search) => {
                search.start();
                self.level = Level::List;
            }
            None => {
                self.message = Some(
                    "Nothing to search on this screen: the summary is one page, not a list of \
                     objects."
                        .to_string(),
                )
            }
        }
    }
}
