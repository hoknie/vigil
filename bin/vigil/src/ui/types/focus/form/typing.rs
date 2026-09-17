use tui_input::{Input, InputRequest};
use vigil_view::Entry;

use super::editing::Editing;

impl Editing {
    pub fn input(&self) -> Option<Input> {
        match self.entry() {
            Some(Entry::Text(text)) => Some(Input::new(text.clone()).with_cursor(self.cursor)),
            _ => None,
        }
    }

    pub fn edit(&mut self, request: InputRequest) {
        let cursor = self.cursor;
        let Some(Entry::Text(text)) = self.entry_mut() else {
            return;
        };
        let mut input = Input::new(std::mem::take(text)).with_cursor(cursor);
        input.handle(request);
        let cursor = input.cursor();
        *text = input.into();
        self.cursor = cursor;
    }

    pub fn type_character(&mut self, character: char) {
        self.edit(InputRequest::InsertChar(character));
    }

    pub fn erase(&mut self) {
        self.edit(InputRequest::DeletePrevChar);
    }
}
