use super::hints::Hints;
use crate::ui::{Level, Screen};

pub(super) fn keys(hints: &Hints<'_>, screen: Screen, width: u16) -> String {
    if hints.typing {
        return " type to search · Enter keep it · Esc put it back".to_string();
    }
    if hints.choosing {
        return match hints.choosing_acts {
            true => " a letter above does it, on this host, now · C or Esc walks away".to_string(),
            false => " ← → choose · Enter apply · Esc leave it as it was".to_string(),
        };
    }

    let long = match hints.level {
        _ if screen == Screen::HOME => match hints.panel {
            true => " j/k ↑↓ a section · → or Enter open it · d close · 1-9 by number · ? keys"
                .to_string(),
            false => " j/k ↑↓ a section · → or Enter open it · d details · 1-9 by number · ? keys"
                .to_string(),
        },
        Level::Menu => format!(
            " ←→ which list · ↓ into it · ↑ or Esc {} · ? keys · q quit",
            hints.back.named()
        ),
        Level::Detail if hints.buttons => format!(
            " j/k ↑↓ scroll · → a button · Enter press it · ← or Esc {} · ? keys",
            BACK_TO_THE_LIST
        ),
        Level::Detail if hints.to_object => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · o object · ? keys".to_string()
        }
        Level::Detail => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · ? keys · q quit".to_string()
        }
        Level::List if screen == Screen::SUMMARY => format!(
            " j/k ↑↓ scroll · d {} · ← or Esc {} · r ask · ? keys",
            match hints.panel {
                true => "hide",
                false => "why",
            },
            hints.back.named()
        ),
        Level::List if hints.panel && hints.marks => {
            listing(hints, Room::Whole, CLOSE_THE_PANEL, Picking::Unsaid)
        }
        Level::List if hints.panel => {
            " j/k ↑↓ move · → detail · / search · ← or Esc close the panel · ? keys".to_string()
        }
        Level::List if screen == Screen::FINDINGS => {
            listing(hints, Room::Whole, hints.back.named(), Picking::AndOne)
        }
        Level::List => listing(hints, Room::Whole, hints.back.named(), Picking::Unsaid),
    };
    let back = match hints.panel {
        true => CLOSE_THE_PANEL,
        false => hints.back.named(),
    };

    let offered = [
        Some(long),
        picking(hints, screen, Picking::Runs),
        picking(hints, screen, Picking::Unsaid),
        short(hints, screen),
        Some(listing(hints, Room::Tight, back, Picking::Unsaid)),
        Some(listing(hints, Room::Cramped, back, Picking::Unsaid)),
    ];
    for line in offered.into_iter().flatten() {
        if line.chars().count() <= width as usize {
            return line;
        }
    }
    LAST_TWO.to_string()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Room {
    Whole,
    Tight,
    Cramped,
}

const LAST_TWO: &str = " ? keys · q quit";

const CLOSE_THE_PANEL: &str = "close the panel";

const BACK_TO_THE_LIST: &str = "back to the list";

fn short(hints: &Hints<'_>, screen: Screen) -> Option<String> {
    match hints.level {
        Level::List if screen == Screen::FINDINGS && !hints.panel => Some(format!(
            " j/k ↑↓ move · → detail · s sort · f filter · ← {} · ? keys",
            hints.back.named()
        )),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Picking {
    AndOne,
    Runs,
    Unsaid,
}

fn picking(hints: &Hints<'_>, screen: Screen, picking: Picking) -> Option<String> {
    match hints.level {
        Level::List if screen == Screen::FINDINGS && !hints.panel => {
            Some(listing(hints, Room::Whole, hints.back.named(), picking))
        }
        _ => None,
    }
}

fn listing(hints: &Hints<'_>, room: Room, back: &str, picking: Picking) -> String {
    let mut line = String::from(" j/k ↑↓ move");
    line.push_str(match picking {
        Picking::AndOne => " · ⇧↑↓ pick · x one",
        Picking::Runs => " · ⇧↑↓ pick",
        Picking::Unsaid => "",
    });
    if room != Room::Cramped {
        line.push_str(" · → detail");
    }
    if let Some(key) = hints.arranges
        && room == Room::Whole
    {
        line.push_str(&format!(" · {key} view"));
    }
    if hints.marks {
        line.push_str(match hints.stops {
            true => " · x mark · K stop",
            false => " · x mark · K close",
        });
        if room == Room::Whole {
            line.push_str(" · S suppress");
        }
    }
    if hints.sorts && room != Room::Cramped {
        line.push_str(" · s sort");
    }
    if hints.filters {
        line.push_str(" · f filter");
    }
    if room == Room::Whole {
        line.push_str(" · / search");
    }
    line.push_str(&format!(" · ← {back} · ? keys"));
    line
}

#[cfg(test)]
mod tests {
    use super::super::hints::Back;
    use super::*;

    fn a_section() -> Screen {
        Screen::parse("ports").expect("a section every build of this console has")
    }

    fn hints(level: Level, back: Back) -> Hints<'static> {
        Hints {
            level,
            back,
            ..Hints::default()
        }
    }

    fn sorting(level: Level, back: Back) -> Hints<'static> {
        Hints {
            sorts: true,
            filters: true,
            ..hints(level, back)
        }
    }

    fn acting(level: Level, back: Back) -> Hints<'static> {
        Hints {
            marks: true,
            ..sorting(level, back)
        }
    }

    #[test]
    fn the_key_that_leaves_a_section_says_where_it_leaves_to() {
        let plain = keys(&hints(Level::List, Back::MainScreen), Screen::FINDINGS, 200);
        let after_a_jump = keys(&hints(Level::List, Back::Finding), Screen::FINDINGS, 200);

        assert!(plain.contains("back to the main screen"), "{plain}");
        assert!(
            after_a_jump.contains("back to the finding"),
            "{after_a_jump}"
        );
    }

    #[test]
    fn the_main_screen_offers_the_numbers_because_that_is_where_they_are_drawn() {
        let home = keys(&hints(Level::List, Back::Nowhere), Screen::HOME, 200);

        assert!(home.contains("1-9"), "{home}");
        assert!(!home.contains("Tab"), "{home}");
    }

    #[test]
    fn the_key_that_shows_what_a_section_says_about_itself_is_on_the_line_and_fits_eighty() {
        let closed = keys(&hints(Level::List, Back::Nowhere), Screen::HOME, 80);
        let open = keys(
            &Hints {
                panel: true,
                ..hints(Level::List, Back::Nowhere)
            },
            Screen::HOME,
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
    fn a_panel_with_buttons_in_it_says_which_keys_walk_them_and_which_presses_one() {
        let line = keys(
            &Hints {
                buttons: true,
                ..hints(Level::Detail, Back::MainScreen)
            },
            a_section(),
            120,
        );

        assert!(line.contains("→ a button"), "{line}");
        assert!(line.contains("Enter press it"), "{line}");
        assert!(
            line.contains("← or Esc back to the list"),
            "and the two keys that go back still go back: {line}"
        );
    }

    #[test]
    fn the_two_keys_that_go_back_are_named_together_because_they_do_the_same_thing() {
        let line = keys(
            &hints(Level::Detail, Back::MainScreen),
            Screen::FINDINGS,
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

        let line = keys(&beside, Screen::FINDINGS, 200);

        assert!(line.contains("← or Esc close the panel"), "{line}");
        assert!(
            !line.contains("back to the main screen"),
            "the rung above is two presses away, not one: {line}"
        );
    }

    #[test]
    fn the_key_that_switches_the_view_is_offered_where_the_list_has_one_to_switch_to() {
        let arranged = keys(
            &Hints {
                arranges: Some('t'),
                ..hints(Level::List, Back::MainScreen)
            },
            a_section(),
            200,
        );
        let plain = keys(&hints(Level::List, Back::MainScreen), a_section(), 200);

        assert!(arranged.contains("t view"), "{arranged}");
        assert!(
            !plain.contains("t view"),
            "the line offers the key the list under the cursor answers to, and a list with \
             one view has no second one to switch to: {plain}"
        );
    }

    #[test]
    fn the_key_that_orders_a_list_is_offered_where_the_list_says_it_can_be_ordered() {
        let sorts = keys(&sorting(Level::List, Back::MainScreen), a_section(), 200);
        let does_not = keys(&hints(Level::List, Back::MainScreen), a_section(), 200);

        assert!(sorts.contains("s sort"), "{sorts}");
        assert!(
            !does_not.contains("s sort"),
            "a key the list answers with a refusal is a key the line must not offer: {does_not}"
        );
    }

    #[test]
    fn the_key_that_walks_to_an_object_is_offered_only_where_there_is_one_to_walk_to() {
        let findings = keys(
            &Hints {
                to_object: true,
                ..hints(Level::Detail, Back::MainScreen)
            },
            Screen::FINDINGS,
            200,
        );
        let a_reading = keys(&hints(Level::Detail, Back::MainScreen), a_section(), 200);

        assert!(findings.contains("o object"), "{findings}");
        assert!(
            !a_reading.contains("o object"),
            "the panel of a reading is already the object: {a_reading}"
        );
    }

    #[test]
    fn the_key_that_picks_a_run_of_findings_is_offered_where_there_is_a_run_to_pick() {
        let findings = keys(&hints(Level::List, Back::MainScreen), Screen::FINDINGS, 200);
        let a_reading = keys(&hints(Level::List, Back::MainScreen), a_section(), 200);

        assert!(findings.contains("⇧↑↓ pick"), "{findings}");
        assert!(
            findings.contains("x one"),
            "the key that picks one row without a modifier is the one a terminal that eats \
             ctrl and shift leaves a reader with: {findings}"
        );
        assert!(
            !a_reading.contains("pick"),
            "only the findings can be picked, and a key offered where it does nothing \
             teaches a reader not to trust the line: {a_reading}"
        );
    }

    #[test]
    fn a_terminal_too_narrow_for_the_line_is_given_the_two_keys_that_matter() {
        let cramped = keys(&hints(Level::List, Back::MainScreen), Screen::FINDINGS, 20);

        assert_eq!(cramped, " ? keys · q quit");
    }

    #[test]
    fn the_two_keys_that_order_and_narrow_a_list_are_on_the_line_and_it_fits_eighty() {
        let findings = keys(
            &sorting(Level::List, Back::MainScreen),
            Screen::FINDINGS,
            80,
        );
        let ports = keys(&sorting(Level::List, Back::MainScreen), a_section(), 80);

        assert_eq!(
            findings, ports,
            "the findings and a list of a reading are the same kind of thing, and a reader \
             who learned the keys on one has learned them on the other"
        );
        for line in [&findings, &ports] {
            assert!(line.contains("s sort"), "{line}");
            assert!(line.contains("f filter"), "{line}");
            assert!(
                line.chars().count() <= 80,
                "{} columns and the whole hint is dropped for the two keys that matter: {line}",
                line.chars().count()
            );
        }
        assert!(
            !findings.contains("s severity"),
            "the severity floor moved into f, and a hint still offering it on s sends a \
             reader to a key that does something else now: {findings}"
        );
    }

    #[test]
    fn a_list_a_reader_can_act_on_says_so_at_the_bottom_even_at_eighty_columns() {
        for width in [80u16, 100, 140, 200] {
            let line = keys(&acting(Level::List, Back::MainScreen), a_section(), width);

            assert!(line.contains("x mark"), "{width}: {line}");
            assert!(line.contains("K close"), "{width}: {line}");
            assert!(
                line.chars().count() <= width as usize,
                "{width}: {} columns: {line}",
                line.chars().count()
            );
        }
    }

    #[test]
    fn a_list_of_programs_says_that_k_stops_them_and_not_that_it_closes_something() {
        let programs = Hints {
            stops: true,
            ..acting(Level::List, Back::MainScreen)
        };

        let line = keys(&programs, a_section(), 120);

        assert!(line.contains("K stop"), "{line}");
        assert!(!line.contains("K close"), "{line}");
    }

    #[test]
    fn a_list_nothing_can_be_done_to_offers_none_of_the_keys_that_do_it() {
        let line = keys(&sorting(Level::List, Back::MainScreen), a_section(), 200);

        for key in ["x mark", "K close", "S suppress"] {
            assert!(
                !line.contains(key),
                "a key the list answers with a refusal is a key the line must not offer: {line}"
            );
        }
    }

    #[test]
    fn the_way_back_is_named_even_on_the_line_that_had_to_be_cut_down_to_fit() {
        for hints in [
            hints(Level::List, Back::Finding),
            sorting(Level::List, Back::Finding),
        ] {
            let line = keys(&hints, a_section(), 80);

            assert!(
                line.contains("back to the finding"),
                "a reader who arrived from a finding is told nothing about the way home: {line}"
            );
            assert!(line.chars().count() <= 80, "{line}");
        }
    }

    #[test]
    fn while_a_choice_is_open_the_line_says_only_the_keys_that_walk_it() {
        let choosing = Hints {
            choosing: true,
            ..hints(Level::List, Back::MainScreen)
        };

        let line = keys(&choosing, Screen::FINDINGS, 80);

        assert!(line.contains("Enter apply"), "{line}");
        assert!(line.contains("Esc leave it as it was"), "{line}");
        assert!(!line.contains("q quit"), "q types nothing here: {line}");
    }
}
