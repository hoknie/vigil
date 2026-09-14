use ratatui::crossterm::event::KeyCode;

use crate::ui::{Editing, Pressed, Spot};

pub fn pressed(editing: &mut Editing, code: KeyCode) -> Pressed {
    match code {
        KeyCode::Esc => Pressed::Leave,
        KeyCode::Enter => match editing.spot() {
            Spot::Back | Spot::Cancel => Pressed::Leave,
            Spot::Save => Pressed::Submit,
            Spot::Field(_) => {
                editing.next();
                Pressed::Nothing
            }
        },
        KeyCode::Down | KeyCode::Tab => {
            editing.next();
            Pressed::Nothing
        }
        KeyCode::Up | KeyCode::BackTab => {
            editing.previous();
            Pressed::Nothing
        }
        KeyCode::Right => {
            editing.sideways(1);
            Pressed::Nothing
        }
        KeyCode::Left => {
            editing.sideways(-1);
            Pressed::Nothing
        }
        KeyCode::Backspace => {
            editing.erase();
            Pressed::Nothing
        }
        KeyCode::Char(' ') if !editing.in_a_text() => {
            editing.toggle();
            Pressed::Nothing
        }
        KeyCode::Char(character) => {
            editing.type_character(character);
            Pressed::Nothing
        }
        _ => Pressed::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Changing;
    use vigil_view::{Field, Form};

    use super::*;
    use crate::ui::Screen;

    fn editing() -> Editing {
        Editing::open(
            Form::new("NEW GROUP")
                .with(Field::text("name", "name", ""))
                .with(Field::switch("system", "system", false)),
            Screen::FINDINGS,
            0,
            None,
            Changing::Create,
        )
    }

    #[test]
    fn escape_leaves_from_anywhere_on_the_form_and_sends_nothing() {
        let mut editing = editing();
        assert_eq!(pressed(&mut editing, KeyCode::Esc), Pressed::Leave);
        editing.next();
        assert_eq!(pressed(&mut editing, KeyCode::Esc), Pressed::Leave);
    }

    #[test]
    fn enter_sends_only_from_save_and_leaves_from_back_and_cancel() {
        let mut editing = editing();
        assert_eq!(
            pressed(&mut editing, KeyCode::Enter),
            Pressed::Nothing,
            "Enter in a field goes on to the next one: a form is not sent by accident from \
             the middle of typing"
        );
        assert_eq!(editing.spot(), Spot::Field(1));

        editing.next();
        assert_eq!(pressed(&mut editing, KeyCode::Enter), Pressed::Submit);
        editing.next();
        assert_eq!(pressed(&mut editing, KeyCode::Enter), Pressed::Leave);

        editing.previous();
        editing.previous();
        editing.previous();
        editing.previous();
        assert_eq!(editing.spot(), Spot::Back);
        assert_eq!(pressed(&mut editing, KeyCode::Enter), Pressed::Leave);
    }

    #[test]
    fn tab_walks_the_form_here_although_it_means_nothing_on_a_list() {
        let mut editing = editing();
        pressed(&mut editing, KeyCode::Tab);
        assert_eq!(editing.spot(), Spot::Field(1));
        pressed(&mut editing, KeyCode::BackTab);
        assert_eq!(editing.spot(), Spot::Field(0));
    }

    #[test]
    fn a_letter_that_is_a_command_on_a_list_is_text_in_a_field() {
        let mut editing = editing();
        for code in [KeyCode::Char('q'), KeyCode::Char('D'), KeyCode::Char(' ')] {
            pressed(&mut editing, code);
        }

        assert_eq!(editing.form().text("name"), Some("qD "));
    }

    #[test]
    fn space_on_a_switch_turns_it_rather_than_typing_a_space() {
        let mut editing = editing();
        editing.next();
        pressed(&mut editing, KeyCode::Char(' '));

        assert_eq!(editing.form().switch("system"), Some(true));
    }
}
