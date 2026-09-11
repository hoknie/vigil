use super::hints::Hints;
use crate::ui::{Level, Screen};

pub(super) fn keys(hints: &Hints<'_>, screen: Screen, width: u16) -> String {
    if hints.typing {
        return " type to search · Enter keep it · Esc put it back".to_string();
    }

    let long = match (hints.level, screen) {
        (_, Screen::Home) => match hints.panel {
            true => " j/k ↑↓ a section · → or Enter open it · d close · 1-9 by number · ? keys"
                .to_string(),
            false => " j/k ↑↓ a section · → or Enter open it · d details · 1-9 by number · ? keys"
                .to_string(),
        },
        (Level::Menu, _) => format!(
            " ←→ which list · ↓ into it · ↑ or Esc {} · ? keys · q quit",
            hints.back.named()
        ),
        (Level::Detail, Screen::Ports | Screen::Programs | Screen::Startup) => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · ? keys · q quit".to_string()
        }
        (Level::Detail, _) => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · o object · ? keys".to_string()
        }
        (Level::List, Screen::Summary) => format!(
            " j/k ↑↓ scroll · d {} · ← or Esc {} · r ask · ? keys",
            match hints.panel {
                true => "hide",
                false => "why",
            },
            hints.back.named()
        ),
        (Level::List, _) if hints.panel => {
            " j/k ↑↓ move · → detail · / search · ← or Esc close the panel · ? keys".to_string()
        }
        (Level::List, Screen::Findings) => format!(
            " j/k ↑↓ move · → detail · o object · / search · s severity · ← {} · ? keys",
            hints.back.named()
        ),
        (Level::List, Screen::Startup) => format!(
            " j/k ↑↓ move · → detail · t view · / search · ← or Esc {} · ? keys",
            hints.back.named()
        ),
        (Level::List, _) => format!(
            " j/k ↑↓ move · → detail · / search · ← or Esc {} · ? keys",
            hints.back.named()
        ),
    };

    match long.chars().count() <= width as usize {
        true => long,
        false => " ? keys · q quit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::hints::Back;
    use super::*;

    fn hints(level: Level, back: Back) -> Hints<'static> {
        Hints {
            typing: false,
            level,
            message: None,
            back,
            panel: false,
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
    fn the_key_that_shows_what_a_section_says_about_itself_is_on_the_line_and_fits_eighty() {
        let closed = keys(&hints(Level::List, Back::Nowhere), Screen::Home, 80);
        let open = keys(
            &Hints {
                panel: true,
                ..hints(Level::List, Back::Nowhere)
            },
            Screen::Home,
            80,
        );

        assert!(closed.contains("d details"), "{closed}");
        assert!(open.contains("d close"), "{open}");
        for line in [&closed, &open] {
            assert!(
                line.chars().count() <= 80,
                "{} columns, so the whole hint is dropped for the two keys that matter: {line}",
                line.chars().count()
            );
            assert!(line.contains("1-9"), "{line}");
        }
    }

    #[test]
    fn the_two_keys_that_go_back_are_named_together_because_they_do_the_same_thing() {
        let line = keys(
            &hints(Level::Detail, Back::MainScreen),
            Screen::Findings,
            200,
        );

        assert!(line.contains("← or Esc back to the list"), "{line}");
        assert!(
            !line.contains("← close it"),
            "one key that closes and another that steps back is the split that was undone: \
             {line}"
        );
    }

    #[test]
    fn the_rung_where_the_panel_is_open_beside_the_list_says_the_key_puts_the_panel_away() {
        let beside = Hints {
            panel: true,
            ..hints(Level::List, Back::MainScreen)
        };

        let line = keys(&beside, Screen::Findings, 200);

        assert!(line.contains("← or Esc close the panel"), "{line}");
        assert!(
            !line.contains("back to the main screen"),
            "the rung above is two presses away, not one: {line}"
        );
    }

    #[test]
    fn the_key_that_switches_the_view_is_offered_where_it_means_something() {
        let startup = keys(&hints(Level::List, Back::MainScreen), Screen::Startup, 200);
        let ports = keys(&hints(Level::List, Back::MainScreen), Screen::Ports, 200);

        assert!(startup.contains("t view"), "{startup}");
        assert!(!ports.contains("t view"), "{ports}");
    }

    #[test]
    fn a_terminal_too_narrow_for_the_line_is_given_the_two_keys_that_matter() {
        let cramped = keys(&hints(Level::List, Back::MainScreen), Screen::Findings, 20);

        assert_eq!(cramped, " ? keys · q quit");
    }
}
