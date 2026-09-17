use vigil_model::Changing;
use vigil_view::{Entry, Form};

use super::dropdown::Dropdown;
use crate::ui::{Screen, Spot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editing {
    form: Form,
    screen: Screen,
    pane: usize,
    row: Option<String>,
    changing: Changing,
    spot: Spot,
    pub(super) cursor: usize,
    pub(super) dropdown: Option<Dropdown>,
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
            cursor: 0,
            dropdown: None,
            trouble: None,
        };
        editing.arrive(editing.stops().get(1).copied().unwrap_or(Spot::Save));
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
            self.arrive(stops[next]);
        }
    }

    fn arrive(&mut self, spot: Spot) {
        self.spot = spot;
        self.dropdown = None;
        self.cursor = match self.entry() {
            Some(Entry::Text(text)) => text.chars().count(),
            _ => 0,
        };
    }

    pub fn focus(&mut self, spot: Spot) {
        if spot != self.spot && self.stops().contains(&spot) {
            self.arrive(spot);
        }
    }

    pub fn next(&mut self) {
        self.step(1);
    }

    pub fn previous(&mut self) {
        self.step(-1);
    }

    pub(super) fn entry(&self) -> Option<&Entry> {
        match self.spot {
            Spot::Field(at) => self.form.fields.get(at).map(|field| &field.entry),
            _ => None,
        }
    }

    pub(super) fn entry_mut(&mut self) -> Option<&mut Entry> {
        match self.spot {
            Spot::Field(at) => self.form.fields.get_mut(at).map(|field| &mut field.entry),
            _ => None,
        }
    }

    pub fn in_a_text(&self) -> bool {
        matches!(self.entry(), Some(Entry::Text(_)))
    }

    pub fn in_a_list(&self) -> bool {
        matches!(self.entry(), Some(Entry::Choices(_)))
    }

    pub fn sideways(&mut self, by: isize) {
        match self.spot {
            Spot::Save if by > 0 => self.spot = Spot::Cancel,
            Spot::Cancel if by < 0 => self.spot = Spot::Save,
            _ => {}
        }
    }

    pub fn toggle(&mut self) {
        if let Some(Entry::Switch(on)) = self.entry_mut() {
            *on = !*on;
        }
    }
}
