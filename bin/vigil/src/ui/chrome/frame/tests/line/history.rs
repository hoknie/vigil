use super::hints::{a_section, hints, sorting};
use crate::ui::Level;
use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::frame::keys::keys;

#[test]
fn a_list_that_keeps_a_history_offers_h_on_a_whole_line_and_gives_it_up_first_when_cut() {
    let with_a_history = Hints {
        histories: true,
        ..sorting(Level::List, Back::MainScreen)
    };

    let wide = keys(&with_a_history, a_section(), 200);
    assert!(wide.contains("H history"), "{wide}");

    let eighty = keys(&with_a_history, a_section(), 80);
    for kept in ["→ detail", "s sort", "f filter", "back to the main screen"] {
        assert!(
            eighty.contains(kept),
            "a line cut to eighty columns keeps the keys that move through the list and the way \
             back, and gives up H first, because H is also a button in the detail of the row: \
             {kept} is missing from {eighty}"
        );
    }
    assert!(!eighty.contains("H history"), "{eighty}");
    assert!(eighty.chars().count() <= 80, "{eighty}");
    let without = keys(&sorting(Level::List, Back::MainScreen), a_section(), 200);
    assert!(
        !without.contains("H history"),
        "a key the list answers with a refusal is a key the line must not offer: {without}"
    );
}

#[test]
fn with_the_history_open_the_line_says_how_to_scroll_it_and_how_to_leave_it() {
    for width in [200u16, 80, 40, 12] {
        let line = keys(
            &Hints {
                history: true,
                histories: true,
                ..hints(Level::List, Back::MainScreen)
            },
            a_section(),
            width,
        );

        assert!(line.contains("Esc back"), "{width}: {line}");
        assert!(
            !line.contains("H history") && !line.contains("s sort"),
            "{width}: the keys of the list underneath do nothing while the panel is open: {line}"
        );
        assert!(line.chars().count() <= width as usize, "{width}: {line}");
    }
}

#[test]
fn the_line_keeps_the_history_where_a_panel_or_a_detail_is_open_because_h_still_opens_it() {
    for (level, panel) in [
        (Level::List, true),
        (Level::Detail, false),
        (Level::Detail, true),
    ] {
        let line = keys(
            &Hints {
                histories: true,
                panel,
                ..sorting(level, Back::MainScreen)
            },
            a_section(),
            100,
        );

        assert!(
            line.contains("H history"),
            "{level:?} with panel {panel}: H opens the history from here as well, and a key \
             that works and is not offered is a key nobody presses: {line}"
        );
    }
}
