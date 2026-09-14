use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::ui::app::KILL;
use crate::ui::{Action, Motion, Screen};
pub fn action(code: KeyCode, modifiers: KeyModifiers, typing: bool) -> Action {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return match code {
            KeyCode::Char('c') => Action::Leave,
            KeyCode::Char('f') => Action::Move(Motion::PageDown),
            KeyCode::Char('b') => Action::Move(Motion::PageUp),
            KeyCode::Down => Action::Gather(Motion::Down),
            KeyCode::Up => Action::Gather(Motion::Up),
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

    if modifiers.contains(KeyModifiers::SHIFT) {
        match code {
            KeyCode::Down => return Action::Pick(Motion::Down),
            KeyCode::Up => return Action::Pick(Motion::Up),
            KeyCode::PageDown => return Action::Pick(Motion::PageDown),
            KeyCode::PageUp => return Action::Pick(Motion::PageUp),
            _ => {}
        }
    }

    match code {
        KeyCode::Char('q') => Action::Leave,
        KeyCode::Esc | KeyCode::Backspace => Action::Back,
        KeyCode::Char('r') | KeyCode::F(5) => Action::Refresh,
        KeyCode::Char('?') => Action::Help,
        KeyCode::F(1) => Action::Help,
        KeyCode::Char(digit @ '1'..='9') => {
            match Screen::all().get(digit as usize - '1' as usize) {
                Some(screen) => Action::Go(*screen),
                None => Action::Ignore,
            }
        }
        KeyCode::Right | KeyCode::Char('l') => Action::Sideways(1),
        KeyCode::Left | KeyCode::Char('h') => Action::Sideways(-1),
        KeyCode::Down | KeyCode::Char('j') => Action::Move(Motion::Down),
        KeyCode::Up | KeyCode::Char('k') => Action::Move(Motion::Up),
        KeyCode::Char('J') => Action::Pick(Motion::Down),
        KeyCode::Char('K') => Action::Pick(Motion::Up),
        KeyCode::PageDown | KeyCode::Char(' ') => Action::Move(Motion::PageDown),
        KeyCode::PageUp => Action::Move(Motion::PageUp),
        KeyCode::Home | KeyCode::Char('g') => Action::Move(Motion::First),
        KeyCode::End | KeyCode::Char('G') => Action::Move(Motion::Last),

        KeyCode::Enter => Action::Open,
        KeyCode::Char('o') => Action::ToObject,
        KeyCode::Char('s') => Action::Sort,
        KeyCode::Char(KILL) => Action::Kill,
        KeyCode::Char('f') => Action::Narrow,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char(letter) => Action::Letter(letter),

        _ => Action::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture::screen;

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
        assert_eq!(plain(KeyCode::Char('1')), Action::Go(screen("ports")));
        assert_eq!(plain(KeyCode::Char('5')), Action::Go(screen("firewall")));
        assert_eq!(plain(KeyCode::Char('7')), Action::Go(screen("containers")));
        assert_eq!(plain(KeyCode::Char('9')), Action::Go(Screen::FINDINGS));
        for digit in 1..=9u8 {
            let key = char::from_digit(u32::from(digit), 10).expect("one of nine");
            assert!(
                matches!(plain(KeyCode::Char(key)), Action::Go(_)),
                "{key} opens nothing, and a number drawn on the main screen that opens \
                 nothing is a key that teaches a reader not to trust the others"
            );
        }
        assert!(
            !matches!(plain(KeyCode::Char('0')), Action::Go(_)),
            "there is no tenth section, and zero is not a way into one"
        );
    }

    #[test]
    fn the_number_drawn_on_a_row_is_the_number_that_opens_it() {
        for screen in Screen::all() {
            let Some(number) = screen.digit() else {
                continue;
            };
            let drawn = char::from_digit(u32::from(number), 10).expect("one of nine");

            assert_eq!(
                plain(KeyCode::Char(drawn)),
                Action::Go(screen),
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
    fn holding_shift_with_an_arrow_picks_a_run_of_rows_instead_of_only_moving() {
        for (code, motion) in [
            (KeyCode::Down, Motion::Down),
            (KeyCode::Up, Motion::Up),
            (KeyCode::PageDown, Motion::PageDown),
            (KeyCode::PageUp, Motion::PageUp),
        ] {
            assert_eq!(
                action(code, KeyModifiers::SHIFT, false),
                Action::Pick(motion)
            );
            assert_eq!(
                plain(code),
                Action::Move(motion),
                "{code:?} on its own moves"
            );
        }
    }

    #[test]
    fn holding_control_with_an_arrow_picks_the_row_it_leaves_and_steps_on() {
        assert_eq!(
            action(KeyCode::Down, KeyModifiers::CONTROL, false),
            Action::Gather(Motion::Down)
        );
        assert_eq!(
            action(KeyCode::Up, KeyModifiers::CONTROL, false),
            Action::Gather(Motion::Up)
        );
    }

    #[test]
    fn the_hand_that_never_leaves_the_letters_picks_a_run_as_well() {
        assert_eq!(plain(KeyCode::Char('J')), Action::Pick(Motion::Down));
        assert_eq!(plain(KeyCode::Char('K')), Action::Pick(Motion::Up));
        assert_eq!(
            action(KeyCode::Char('J'), KeyModifiers::SHIFT, false),
            Action::Pick(Motion::Down),
            "a terminal that reports the shift beside the capital must not mean something else"
        );
    }

    #[test]
    fn the_capitals_a_reading_answers_to_are_still_letters_and_not_a_run_of_rows() {
        for letter in ['T', 'U', 'G'] {
            assert!(
                !matches!(
                    action(KeyCode::Char(letter), KeyModifiers::SHIFT, false),
                    Action::Pick(_)
                ),
                "{letter} was taken over by picking"
            );
        }
    }

    #[test]
    fn shift_and_control_type_nothing_into_a_search_box_that_is_open() {
        assert_eq!(
            action(KeyCode::Down, KeyModifiers::SHIFT, true),
            Action::Move(Motion::Down),
            "a reader in the middle of typing is moving the list under it, not picking rows"
        );
    }

    #[test]
    fn every_key_this_console_ignores_says_so_rather_than_falling_through() {
        assert_eq!(plain(KeyCode::Insert), Action::Ignore);
        assert_eq!(typed(KeyCode::Insert), Action::Ignore);
    }
}
