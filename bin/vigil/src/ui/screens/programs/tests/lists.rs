use super::harness::drawn;
use crate::ui::{Program, fixture};

#[test]
fn each_list_answers_for_its_own_collector_and_never_for_a_neighbour() {
    let view = fixture::view();

    let launches = drawn(&view, Program::Launches, 80);

    assert!(
        launches.contains("not reading what people run"),
        "the launches collector is off in this fixture and the list says so: {launches}"
    );
    assert!(
        !launches.contains("nginx"),
        "and it does not borrow the other list's reading: {launches}"
    );
}

#[test]
fn a_list_whose_collector_is_switched_off_names_the_key_that_turns_it_on() {
    let page = drawn(&fixture::view(), Program::Launches, 80);

    assert!(page.contains("collectors:"), "{page}");
    assert!(page.contains("not failing"), "{page}");
}

#[test]
fn a_launch_says_the_file_was_there_when_the_record_was_read_and_not_that_it_is_there_now() {
    let page = drawn(&fixture::view_with_launches(), Program::Launches, 80);

    assert!(page.contains("nmap"), "{page}");
    assert!(page.contains("payload"), "{page}");
    assert!(
        page.contains("never removed from it"),
        "a list that never loses a row has to say so: {page}"
    );
}

#[test]
fn the_row_that_says_the_spool_was_dropping_is_a_row_with_a_key_of_its_own() {
    let page = drawn(&fixture::view_with_launches(), Program::Launches, 80);

    assert!(page.contains("dropped the oldest"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for program in Program::ALL {
            let page = drawn(&fixture::view_with_launches(), *program, width);
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}
