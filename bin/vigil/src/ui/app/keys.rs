use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::App;

use crate::ui::helpers::motion::keys;
use crate::ui::screens::home;
use crate::ui::{Action, Level, Screen};

impl App {
    pub fn on_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let action = keys::action(code, modifiers, self.typing());

        if self.helping {
            self.helping = false;
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.leaving = true;
            }
            return;
        }

        self.message = None;
        self.act(action);
        self.settle();
    }

    pub(super) fn act(&mut self, action: Action) {
        match action {
            Action::Leave => self.leaving = true,
            Action::Back => self.go_back(),
            Action::Refresh => self.refresh_wanted = true,
            Action::Go(screen) => self.visit(screen),
            Action::Sideways(by) => self.sideways(by),
            Action::Move(motion) => self.move_within(motion),
            Action::Open => self.open(),
            Action::ToObject => self.jump_to_object(),
            Action::Floor(by) => match self.nav.at() {
                Screen::Findings => {
                    self.level = Level::List;
                    self.filter.move_floor(by);
                }
                _ => self.message = Some(elsewhere("The severity floor", Screen::Findings)),
            },
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

    fn searching(&mut self) {
        if self.nav.at() == Screen::Home {
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

fn elsewhere(what: &str, screen: Screen) -> String {
    match screen.digit() {
        Some(number) => format!(
            "{what} belongs to the {} section: press {number}.",
            screen.name()
        ),
        None => format!("{what} belongs to the {} section.", screen.name()),
    }
}
