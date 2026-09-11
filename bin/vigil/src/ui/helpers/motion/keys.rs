use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::ui::{Action, Motion, Screen};
pub fn action(code: KeyCode, modifiers: KeyModifiers, typing: bool) -> Action {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return match code {
            KeyCode::Char('c') => Action::Leave,
            KeyCode::Char('f') => Action::Move(Motion::PageDown),
            KeyCode::Char('b') => Action::Move(Motion::PageUp),
            _ => Action::Ignore,
        };
    }

    if typing {
        return match code {
            KeyCode::Esc => Action::Abandon,
            KeyCode::Enter => Action::Accept,
            KeyCode::Backspace => Action::Erase,
            KeyCode::Down => Action::Move(Motion::Down),
            KeyCode::Up => Action::Move(Motion::Up),
            KeyCode::PageDown => Action::Move(Motion::PageDown),
            KeyCode::PageUp => Action::Move(Motion::PageUp),
            KeyCode::Char(character) => Action::Type(character),
            _ => Action::Ignore,
        };
    }

    match code {
        KeyCode::Char('q') => Action::Leave,
        KeyCode::Esc | KeyCode::Backspace => Action::Back,
        KeyCode::Char('r') | KeyCode::F(5) => Action::Refresh,
        KeyCode::Char('?') => Action::Help,
        KeyCode::F(1) => Action::Help,
        KeyCode::Char(digit @ '1'..='9') => match Screen::ALL.get(digit as usize - '1' as usize) {
            Some(screen) => Action::Go(*screen),
            None => Action::Ignore,
        },
        KeyCode::Right | KeyCode::Char('l') => Action::Sideways(1),
        KeyCode::Left | KeyCode::Char('h') => Action::Sideways(-1),
        KeyCode::Down | KeyCode::Char('j') => Action::Move(Motion::Down),
        KeyCode::Up | KeyCode::Char('k') => Action::Move(Motion::Up),
        KeyCode::PageDown | KeyCode::Char(' ') => Action::Move(Motion::PageDown),
        KeyCode::PageUp => Action::Move(Motion::PageUp),
        KeyCode::Home | KeyCode::Char('g') => Action::Move(Motion::First),
        KeyCode::End | KeyCode::Char('G') => Action::Move(Motion::Last),

        KeyCode::Enter => Action::Open,
        KeyCode::Char('o') => Action::ToObject,
        KeyCode::Char('s') => Action::Floor(1),
        KeyCode::Char('S') => Action::Floor(-1),
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char(letter) => Action::Letter(letter),

        _ => Action::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(code: KeyCode) -> Action {
        action(code, KeyModifiers::NONE, false)
    }

    fn typed(code: KeyCode) -> Action {
        action(code, KeyModifiers::NONE, true)
    }

    #[test]
    fn a_letter_typed_into_the_search_box_is_text_and_not_a_command() {
        assert_eq!(typed(KeyCode::Char('q')), Action::Type('q'));
        assert_eq!(typed(KeyCode::Char('/')), Action::Type('/'));
        assert_eq!(typed(KeyCode::Char('4')), Action::Type('4'));
        assert_eq!(plain(KeyCode::Char('q')), Action::Leave);
    }

    #[test]
    fn control_c_leaves_from_inside_the_search_box_as_well() {
        for typing in [false, true] {
            assert_eq!(
                action(KeyCode::Char('c'), KeyModifiers::CONTROL, typing),
                Action::Leave
            );
        }
    }

    #[test]
    fn the_digits_name_the_sections_in_the_order_the_main_screen_lists_them() {
        assert_eq!(plain(KeyCode::Char('1')), Action::Go(Screen::Ports));
        assert_eq!(plain(KeyCode::Char('6')), Action::Go(Screen::Findings));
        assert_eq!(
            plain(KeyCode::Char('9')),
            Action::Ignore,
            "a digit with no section behind it does nothing rather than something else"
        );
    }

    #[test]
    fn the_number_drawn_on_a_row_is_the_number_that_opens_it() {
        for screen in Screen::ALL {
            let Some(number) = screen.digit() else {
                continue;
            };
            let drawn = char::from_digit(u32::from(number), 10).expect("one of nine");

            assert_eq!(
                plain(KeyCode::Char(drawn)),
                Action::Go(*screen),
                "the main screen draws {number} on {} and {number} opens something else",
                screen.name()
            );
        }
    }

    #[test]
    fn both_hands_reach_the_same_movements() {
        assert_eq!(plain(KeyCode::Char('j')), plain(KeyCode::Down));
        assert_eq!(plain(KeyCode::Char('k')), plain(KeyCode::Up));
        assert_eq!(plain(KeyCode::Char('g')), plain(KeyCode::Home));
        assert_eq!(plain(KeyCode::Char('G')), plain(KeyCode::End));
        assert_eq!(plain(KeyCode::Char('l')), plain(KeyCode::Right));
        assert_eq!(plain(KeyCode::Char('h')), plain(KeyCode::Left));
        assert_eq!(
            action(KeyCode::Char('f'), KeyModifiers::CONTROL, false),
            plain(KeyCode::PageDown)
        );
    }

    #[test]
    fn the_horizontal_arrows_go_sideways_and_the_tab_key_means_nothing_at_all() {
        assert_eq!(plain(KeyCode::Right), Action::Sideways(1));
        assert_eq!(plain(KeyCode::Left), Action::Sideways(-1));
        assert_eq!(
            plain(KeyCode::Tab),
            Action::Ignore,
            "it walked a row of tabs that no longer exists; a key that means roughly \
             something is worse than a key that means nothing"
        );
        assert_eq!(plain(KeyCode::BackTab), Action::Ignore);
    }

    #[test]
    fn escape_steps_back_and_does_not_leave_on_its_own() {
        assert_eq!(plain(KeyCode::Esc), Action::Back);
        assert_eq!(plain(KeyCode::Backspace), Action::Back);
    }

    #[test]
    fn every_key_this_console_ignores_says_so_rather_than_falling_through() {
        assert_eq!(plain(KeyCode::Insert), Action::Ignore);
        assert_eq!(typed(KeyCode::Insert), Action::Ignore);
    }
}
