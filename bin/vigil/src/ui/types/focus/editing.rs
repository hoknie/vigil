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

#[cfg(test)]
mod tests {
    use vigil_view::{Choice, Field};

    use super::*;

    fn editing() -> Editing {
        Editing::open(
            Form::new("EDIT THE ACCOUNT deploy")
                .with(Field::fixed("name", "name", "deploy"))
                .with(Field::text("shell", "shell", "/bin/bash"))
                .with(Field::switch("locked", "locked", false))
                .with(Field::choices(
                    "groups",
                    "groups",
                    vec![Choice::of("wheel", true), Choice::of("docker", false)],
                )),
            Screen::FINDINGS,
            0,
            Some("account|deploy".into()),
            Changing::Update,
        )
    }

    #[test]
    fn a_form_opens_on_its_first_field_a_person_can_change_and_not_on_the_name_it_is_about() {
        assert_eq!(editing().spot(), Spot::Field(1));
    }

    #[test]
    fn the_arrows_walk_from_back_through_the_fields_to_save_and_cancel_and_stop_at_both_ends() {
        let mut editing = editing();

        editing.previous();
        assert_eq!(editing.spot(), Spot::Back);
        editing.previous();
        assert_eq!(
            editing.spot(),
            Spot::Back,
            "the way out at the top does not wrap round to the bottom of the form"
        );

        for wanted in [
            Spot::Field(1),
            Spot::Field(2),
            Spot::Field(3),
            Spot::Save,
            Spot::Cancel,
            Spot::Cancel,
        ] {
            editing.next();
            assert_eq!(editing.spot(), wanted);
        }
    }

    #[test]
    fn a_fixed_field_is_passed_over_because_nothing_in_it_can_be_changed() {
        let mut editing = editing();
        editing.previous();
        editing.next();

        assert_ne!(editing.spot(), Spot::Field(0));
    }

    #[test]
    fn typing_and_erasing_change_the_text_field_the_focus_is_on_and_nothing_else() {
        let mut editing = editing();
        editing.erase();
        editing.erase();
        editing.erase();
        editing.erase();
        for character in "sh".chars() {
            editing.type_character(character);
        }

        assert_eq!(editing.form().text("shell"), Some("/bin/sh"));
        assert!(editing.form().changed("shell"));
        assert!(!editing.form().changed("locked"));
    }

    #[test]
    fn space_turns_a_switch_and_the_choice_under_the_cursor_and_the_side_arrows_walk_choices() {
        let mut editing = editing();
        editing.next();
        editing.toggle();
        assert_eq!(editing.form().switch("locked"), Some(true));

        editing.next();
        editing.sideways(1);
        editing.sideways(1);
        assert_eq!(
            editing.choice(),
            1,
            "the last choice is where the arrow stops"
        );
        editing.toggle();
        assert_eq!(
            editing.form().chosen("groups"),
            Some(vec!["wheel", "docker"])
        );
    }

    #[test]
    fn the_side_arrows_walk_between_save_and_cancel() {
        let mut editing = editing();
        for _ in 0..3 {
            editing.next();
        }
        assert_eq!(editing.spot(), Spot::Save);

        editing.sideways(1);
        assert_eq!(editing.spot(), Spot::Cancel);
        editing.sideways(-1);
        assert_eq!(editing.spot(), Spot::Save);
    }

    #[test]
    fn a_letter_typed_on_a_button_types_nothing() {
        let mut editing = editing();
        editing.previous();
        editing.type_character('x');

        assert_eq!(editing.form().text("shell"), Some("/bin/bash"));
    }

    #[test]
    fn what_went_wrong_stays_on_the_form_beside_what_was_typed() {
        let mut editing = editing();
        editing.type_character('x');
        editing.said("usermod refused");

        assert_eq!(editing.trouble(), Some("usermod refused"));
        assert_eq!(editing.form().text("shell"), Some("/bin/bashx"));
    }
}
