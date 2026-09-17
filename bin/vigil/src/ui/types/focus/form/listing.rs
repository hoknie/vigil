use vigil_view::{Choice, Entry};

use super::dropdown::Dropdown;
use super::editing::Editing;

impl Editing {
    pub fn dropdown(&self) -> Option<&Dropdown> {
        self.dropdown.as_ref()
    }

    pub fn choices(&self) -> &[Choice] {
        match self.entry() {
            Some(Entry::Choices(choices)) => choices,
            _ => &[],
        }
    }

    pub fn open_the_list(&mut self) {
        if self.in_a_list() {
            self.dropdown = Some(Dropdown::default());
        }
    }

    pub fn close_the_list(&mut self) {
        self.dropdown = None;
    }

    pub fn walk_the_list(&mut self, by: isize) {
        let Some(mut dropdown) = self.dropdown.take() else {
            return;
        };
        dropdown.step(by, self.choices());
        self.dropdown = Some(dropdown);
    }

    pub fn narrow_the_list(&mut self, character: char) {
        if let Some(dropdown) = self.dropdown.as_mut() {
            dropdown.narrow(character);
        }
    }

    pub fn widen_the_list(&mut self) {
        if let Some(dropdown) = self.dropdown.as_mut() {
            dropdown.widen();
        }
    }

    pub fn toggle_in_the_list(&mut self) {
        let Some(at) = self
            .dropdown
            .as_ref()
            .and_then(|dropdown| dropdown.under_the_cursor(self.choices()))
        else {
            return;
        };
        if let Some(Entry::Choices(choices)) = self.entry_mut()
            && let Some(one) = choices.get_mut(at)
        {
            one.chosen = !one.chosen;
        }
    }
}
