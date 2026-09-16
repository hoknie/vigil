use super::hints::{a_section, hints, sorting};
use crate::ui::Level;
use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::frame::keys::keys;

#[test]
fn a_list_that_keeps_a_history_offers_h_on_the_line_at_every_width_down_to_eighty() {
    for width in [80u16, 100, 140, 200] {
        let line = keys(
            &Hints {
                histories: true,
                ..sorting(Level::List, Back::MainScreen)
            },
            a_section(),
            width,
        );

        assert!(line.contains("H history"), "{width}: {line}");
        assert!(line.contains("f filter"), "{width}: {line}");
        assert!(line.chars().count() <= width as usize, "{width}: {line}");
    }
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
