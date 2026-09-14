use vigil_model::Changing;
use vigil_view::{Entry, Form};

use crate::ui::{Screen, Spot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editing {
    form: Form,
    screen: Screen,
    pane: usize,
    row: Option<String>,
    changing: Changing,
    spot: Spot,
    choice: usize,
    trouble: Option<String>,
}

impl Editing {
    pub fn open(
        form: Form,
        screen: Screen,
        pane: usize,
        row: Option<String>,
        changing: Changing,
    ) -> Editing {
        let mut editing = Editing {
            form,
            screen,
            pane,
            row,
            changing,
            spot: Spot::Back,
            choice: 0,
            trouble: None,
        };
        editing.spot = editing.stops().get(1).copied().unwrap_or(Spot::Save);
        editing
    }

    pub fn form(&self) -> &Form {
        &self.form
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn pane(&self) -> usize {
        self.pane
    }

    pub fn row(&self) -> Option<&str> {
        self.row.as_deref()
    }

    pub fn changing(&self) -> Changing {
        self.changing
    }

    pub fn spot(&self) -> Spot {
        self.spot
    }

    pub fn choice(&self) -> usize {
        self.choice
    }

    pub fn trouble(&self) -> Option<&str> {
        self.trouble.as_deref()
    }

    pub fn said(&mut self, trouble: impl Into<String>) {
        self.trouble = Some(trouble.into());
    }

    fn stops(&self) -> Vec<Spot> {
        let mut stops = vec![Spot::Back];
        stops.extend(
            self.form
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| field.editable())
                .map(|(at, _)| Spot::Field(at)),
        );
        stops.push(Spot::Save);
        stops.push(Spot::Cancel);
        stops
    }

    fn step(&mut self, by: isize) {
        let stops = self.stops();
        let here = stops
            .iter()
            .position(|stop| *stop == self.spot)
            .unwrap_or(0) as isize;
        let next = (here + by).clamp(0, stops.len() as isize - 1) as usize;
        if stops[next] != self.spot {
            self.spot = stops[next];
            self.choice = 0;
        }
    }

    pub fn next(&mut self) {
        self.step(1);
    }

    pub fn previous(&mut self) {
        self.step(-1);
    }

    fn entry(&self) -> Option<&Entry> {
        match self.spot {
            Spot::Field(at) => self.form.fields.get(at).map(|field| &field.entry),
            _ => None,
        }
    }

    fn entry_mut(&mut self) -> Option<&mut Entry> {
        match self.spot {
            Spot::Field(at) => self.form.fields.get_mut(at).map(|field| &mut field.entry),
            _ => None,
        }
    }

    pub fn in_a_text(&self) -> bool {
        matches!(self.entry(), Some(Entry::Text(_)))
    }

    pub fn sideways(&mut self, by: isize) {
        match (self.spot, self.entry()) {
            (Spot::Save, _) if by > 0 => self.spot = Spot::Cancel,
            (Spot::Cancel, _) if by < 0 => self.spot = Spot::Save,
            (_, Some(Entry::Choices(choices))) if !choices.is_empty() => {
                let last = choices.len() as isize - 1;
                self.choice = (self.choice as isize + by).clamp(0, last) as usize;
            }
            _ => {}
        }
    }

    pub fn toggle(&mut self) {
        let choice = self.choice;
        match self.entry_mut() {
            Some(Entry::Switch(on)) => *on = !*on,
            Some(Entry::Choices(choices)) => {
                if let Some(one) = choices.get_mut(choice) {
                    one.chosen = !one.chosen;
                }
            }
            _ => {}
        }
    }

    pub fn type_character(&mut self, character: char) {
        if let Some(Entry::Text(text)) = self.entry_mut() {
            text.push(character);
        }
    }

    pub fn erase(&mut self) {
        if let Some(Entry::Text(text)) = self.entry_mut() {
            text.pop();
        }
    }
}
