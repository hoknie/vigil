use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::to_input_request;

use crate::ui::{Editing, Pressed, Spot};

pub fn pressed(editing: &mut Editing, code: KeyCode, modifiers: KeyModifiers) -> Pressed {
    if editing.dropdown().is_some() {
        in_the_list(editing, code, modifiers);
        return Pressed::Nothing;
    }
    match code {
        KeyCode::Esc => Pressed::Leave,
        KeyCode::Enter => match editing.spot() {
            Spot::Back | Spot::Cancel => Pressed::Leave,
            Spot::Save => Pressed::Submit,
            Spot::Field(_) if editing.in_a_list() => {
                editing.open_the_list();
                Pressed::Nothing
            }
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
        KeyCode::Backspace if editing.in_a_text() && modifiers.is_empty() => {
            editing.erase();
            Pressed::Nothing
        }
        KeyCode::Char(character)
            if editing.in_a_text()
                && !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            editing.type_character(character);
            Pressed::Nothing
        }
        _ if editing.in_a_text() => {
            if let Some(request) = to_input_request(&Event::Key(KeyEvent::new(code, modifiers))) {
                editing.edit(request);
            }
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
        KeyCode::Char(' ') if editing.in_a_list() => {
            editing.open_the_list();
            Pressed::Nothing
        }
        KeyCode::Char(' ') => {
            editing.toggle();
            Pressed::Nothing
        }
        _ => Pressed::Nothing,
    }
}

fn in_the_list(editing: &mut Editing, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Esc | KeyCode::Enter => editing.close_the_list(),
        KeyCode::Tab => editing.next(),
        KeyCode::BackTab => editing.previous(),
        KeyCode::Down => editing.walk_the_list(1),
        KeyCode::Up => editing.walk_the_list(-1),
        KeyCode::PageDown => editing.walk_the_list(PAGE),
        KeyCode::PageUp => editing.walk_the_list(-PAGE),
        KeyCode::Home => editing.walk_the_list(-isize::MAX),
        KeyCode::End => editing.walk_the_list(isize::MAX),
        KeyCode::Char(' ') => editing.toggle_in_the_list(),
        KeyCode::Backspace => editing.widen_the_list(),
        KeyCode::Char(character) if !modifiers.contains(KeyModifiers::CONTROL) => {
            editing.narrow_the_list(character)
        }
        _ => {}
    }
}

const PAGE: isize = 8;

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
        assert_eq!(
            pressed(&mut editing, KeyCode::Esc, KeyModifiers::NONE),
            Pressed::Leave
        );
        editing.next();
        assert_eq!(
            pressed(&mut editing, KeyCode::Esc, KeyModifiers::NONE),
            Pressed::Leave
        );
    }

    #[test]
    fn enter_sends_only_from_save_and_leaves_from_back_and_cancel() {
        let mut editing = editing();
        assert_eq!(
            pressed(&mut editing, KeyCode::Enter, KeyModifiers::NONE),
            Pressed::Nothing,
            "Enter in a field goes on to the next one: a form is not sent by accident from \
             the middle of typing"
        );
        assert_eq!(editing.spot(), Spot::Field(1));

        editing.next();
        assert_eq!(
            pressed(&mut editing, KeyCode::Enter, KeyModifiers::NONE),
            Pressed::Submit
        );
        editing.next();
        assert_eq!(
            pressed(&mut editing, KeyCode::Enter, KeyModifiers::NONE),
            Pressed::Leave
        );

        editing.previous();
        editing.previous();
        editing.previous();
        editing.previous();
        assert_eq!(editing.spot(), Spot::Back);
        assert_eq!(
            pressed(&mut editing, KeyCode::Enter, KeyModifiers::NONE),
            Pressed::Leave
        );
    }

    #[test]
    fn tab_walks_the_form_here_although_it_means_nothing_on_a_list() {
        let mut editing = editing();
        pressed(&mut editing, KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(editing.spot(), Spot::Field(1));
        pressed(&mut editing, KeyCode::BackTab, KeyModifiers::NONE);
        assert_eq!(editing.spot(), Spot::Field(0));
    }

    #[test]
    fn a_letter_that_is_a_command_on_a_list_is_text_in_a_field() {
        let mut editing = editing();
        for code in [KeyCode::Char('q'), KeyCode::Char('D'), KeyCode::Char(' ')] {
            pressed(&mut editing, code, KeyModifiers::NONE);
        }

        assert_eq!(editing.form().text("name"), Some("qD "));
    }

    #[test]
    fn space_on_a_switch_turns_it_rather_than_typing_a_space() {
        let mut editing = editing();
        editing.next();
        pressed(&mut editing, KeyCode::Char(' '), KeyModifiers::NONE);

        assert_eq!(editing.form().switch("system"), Some(true));
    }

    fn with_a_shell() -> Editing {
        Editing::open(
            Form::new("EDIT").with(Field::text("shell", "shell", "/usr/bin/fish")),
            Screen::FINDINGS,
            0,
            None,
            Changing::Update,
        )
    }

    #[test]
    fn the_side_arrows_home_and_end_move_the_cursor_inside_a_text_and_a_letter_lands_there() {
        let mut editing = with_a_shell();
        pressed(&mut editing, KeyCode::Home, KeyModifiers::NONE);
        pressed(&mut editing, KeyCode::Right, KeyModifiers::NONE);
        pressed(&mut editing, KeyCode::Char('~'), KeyModifiers::NONE);
        pressed(&mut editing, KeyCode::End, KeyModifiers::NONE);
        pressed(&mut editing, KeyCode::Left, KeyModifiers::NONE);
        pressed(&mut editing, KeyCode::Delete, KeyModifiers::NONE);

        assert_eq!(editing.form().text("shell"), Some("/~usr/bin/fis"));
    }

    #[test]
    fn control_w_takes_the_word_before_the_cursor_and_control_u_the_whole_line() {
        let mut editing = with_a_shell();
        pressed(&mut editing, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(
            editing.form().text("shell"),
            Some("/usr/bin/"),
            "the word keys a shell user reaches for work in a field as they do at a prompt"
        );

        pressed(&mut editing, KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert_eq!(editing.form().text("shell"), Some(""));
    }

    #[test]
    fn a_capital_typed_with_shift_is_a_letter_and_a_control_chord_types_nothing() {
        let mut editing = with_a_shell();
        pressed(&mut editing, KeyCode::Char('X'), KeyModifiers::SHIFT);
        pressed(&mut editing, KeyCode::Char('z'), KeyModifiers::CONTROL);

        assert_eq!(editing.form().text("shell"), Some("/usr/bin/fishX"));
    }
}
