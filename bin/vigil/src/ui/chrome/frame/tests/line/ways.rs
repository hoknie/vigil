use super::hints::{a_section, hints, sorting};
use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::frame::keys::keys;
use crate::ui::{Level, Screen};

#[test]
fn a_form_says_which_keys_walk_it_and_that_esc_goes_back_at_every_width() {
    for width in [40u16, 80, 120] {
        let line = keys(
            &Hints {
                editing: true,
                ..hints(Level::List, Back::MainScreen)
            },
            a_section(),
            width,
        );

        assert!(line.contains("Esc back"), "{width}: {line}");
        assert!(line.chars().count() <= width as usize, "{width}: {line}");
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
fn a_terminal_too_narrow_for_the_line_is_given_the_two_keys_that_matter() {
    let cramped = keys(&hints(Level::List, Back::MainScreen), Screen::FINDINGS, 20);

    assert_eq!(cramped, " ? keys · q quit");
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
