use super::hints::{Back, Hints};
use crate::ui::{Level, Screen};

const ROW_OF_LISTS: &str = "the row of lists";

pub(super) fn keys(hints: &Hints<'_>, screen: Screen, width: u16) -> String {
    if hints.typing {
        return " type to search · Enter keep it · Esc put it back".to_string();
    }

    let long = match (hints.level, screen) {
        (_, Screen::Home) => {
            " j/k ↑↓ a section · → or Enter open it · 1-9 open one by number · ? keys · q quit"
                .to_string()
        }
        (Level::Menu, _) => format!(
            " ←→ which list · ↓ into it · ↑ or Esc {} · ? keys · q quit",
            hints.back.named()
        ),
        (Level::Detail, Screen::Ports | Screen::Programs | Screen::Startup) => {
            " j/k ↑↓ PgUp/PgDn scroll · ← close it · Esc back to the list · ? keys · q quit"
                .to_string()
        }
        (Level::Detail, _) => {
            " j/k ↑↓ PgUp/PgDn scroll · ← close it · Esc back to the list · o object · ? keys"
                .to_string()
        }
        (Level::List, Screen::Findings) => format!(
            " j/k ↑↓ move · → detail · o object · / search · s severity · Esc {} · ? keys",
            hints.back.named()
        ),
        (Level::List, Screen::Summary) => format!(
            " j/k ↑↓ scroll · Esc {} · r refresh · ? keys · q quit",
            hints.back.named()
        ),
        (Level::List, _) => match hints.back {
            Back::Finding => format!(
                " j/k ↑↓ move · → detail · / search · ← {ROW_OF_LISTS} · Esc {} · ? keys",
                hints.back.named()
            ),
            _ => format!(" j/k ↑↓ move · → detail · / search · ← or Esc {ROW_OF_LISTS} · ? keys"),
        },
    };

    match long.chars().count() <= width as usize {
        true => long,
        false => " ? keys · q quit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hints(level: Level, back: Back) -> Hints<'static> {
        Hints {
            typing: false,
            level,
            message: None,
            back,
        }
    }

    #[test]
    fn the_key_that_leaves_a_section_says_where_it_leaves_to() {
        let plain = keys(&hints(Level::List, Back::MainScreen), Screen::Findings, 200);
        let after_a_jump = keys(&hints(Level::List, Back::Finding), Screen::Findings, 200);

        assert!(plain.contains("back to the main screen"), "{plain}");
        assert!(
            after_a_jump.contains("back to the finding"),
            "{after_a_jump}"
        );
    }

    #[test]
    fn the_main_screen_offers_the_numbers_because_that_is_where_they_are_drawn() {
        let home = keys(&hints(Level::List, Back::Nowhere), Screen::Home, 200);

        assert!(home.contains("1-9"), "{home}");
        assert!(!home.contains("Tab"), "{home}");
    }

    #[test]
    fn a_terminal_too_narrow_for_the_line_is_given_the_two_keys_that_matter() {
        let cramped = keys(&hints(Level::List, Back::MainScreen), Screen::Findings, 20);

        assert_eq!(cramped, " ? keys · q quit");
    }
}
