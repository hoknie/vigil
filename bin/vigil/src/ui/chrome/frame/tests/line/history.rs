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
