use vigil_model::{Changing, ControlTarget};

use super::hints::{a_section, acting, hints, sorting};
use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::frame::keys::{LAST_TWO, keys};
use crate::ui::{Level, Screen};

#[test]
fn a_list_of_accounts_offers_the_keys_that_change_them_and_not_the_key_that_kills() {
    for width in [80u16, 120, 200] {
        let line = keys(
            &Hints {
                marks: true,
                changes: &[Changing::Create, Changing::Update, Changing::Delete],
                ..hints(Level::List, Back::MainScreen)
            },
            a_section(),
            width,
        );

        assert!(!line.contains("K "), "{width}: {line}");
        assert!(
            line.contains('n') && line.contains('e') && line.contains('D'),
            "{width}: {line}"
        );
        assert!(line.chars().count() <= width as usize, "{width}: {line}");
        assert_ne!(
            line, LAST_TWO,
            "{width}: the keys that change a row were dropped"
        );
    }
    let wide = keys(
        &Hints {
            marks: true,
            changes: &[Changing::Update, Changing::Delete],
            ..hints(Level::List, Back::MainScreen)
        },
        a_section(),
        200,
    );
    assert!(wide.contains("e edit · D delete"), "{wide}");
    assert!(
        !wide.contains("n new"),
        "a list that creates nothing does not offer n: {wide}"
    );
}

#[test]
fn a_list_of_what_this_host_starts_by_itself_offers_the_key_that_acts_on_it_at_eighty_columns() {
    for (target, word) in [
        (ControlTarget::Unit, "U stop/disable"),
        (ControlTarget::Cron, "U comment out"),
    ] {
        for width in [80u16, 120, 200] {
            let line = keys(
                &Hints {
                    marks: true,
                    controls: Some(target),
                    ..hints(Level::List, Back::MainScreen)
                },
                a_section(),
                width,
            );

            assert!(
                line.contains('U'),
                "{width}: the key is dropped on the list it is the only key of: {line}"
            );
            assert!(!line.contains("K "), "{width}: {line}");
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
        let wide = keys(
            &Hints {
                marks: true,
                controls: Some(target),
                ..hints(Level::List, Back::MainScreen)
            },
            a_section(),
            200,
        );
        assert!(wide.contains(word), "{wide}");
    }
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

    for key in ["x mark", "K close", "S suppress", "U stop/disable"] {
        assert!(
            !line.contains(key),
            "a key the list answers with a refusal is a key the line must not offer: {line}"
        );
    }
}
