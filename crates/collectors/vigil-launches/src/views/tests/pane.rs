use vigil_view::{Facet, Pane, Room, Section, Showing, Sorting, conformance};

use crate::fixture::launches;
use crate::views::WhatHasRunHere;

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

const ABOUT_THE_READING: &str = "launches|";

#[test]
fn the_pane_of_this_module_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &launches());
}

#[test]
fn this_module_puts_its_list_in_the_section_the_processes_also_write_to() {
    assert_eq!(WhatHasRunHere.name(), "programs");
    assert_eq!(pane().name(), "launches");
}

#[test]
fn a_launch_is_shown_under_the_person_who_ran_it() {
    let reading = launches();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("run|"))
        .expect("the sample holds a launch");

    let cells = pane.cells(&reading, &row, Room::of(160));

    assert_eq!(cells.len(), 6);
    assert!(!cells[0].text.is_empty(), "who ran it: {cells:?}");
}

#[test]
fn a_launch_says_how_many_times_it_was_run_even_at_eighty_columns() {
    let reading = launches();
    let pane = pane();

    let cells = pane.cells(
        &reading,
        &vigil_view::RowKey::of("run|alice|/usr/bin/nc.openbsd"),
        Room::of(80),
    );

    assert_eq!(
        cells[2].text, "2",
        "alice ran nc twice in the sample, and a row that says only that she ran it once \
         hides the difference between a mistake and a habit: {cells:?}"
    );
}

#[test]
fn sorted_by_runs_the_most_run_program_comes_first_and_the_reading_itself_stays_on_top() {
    let reading = launches();
    let pane = pane();
    let most_first = Sorting::of(
        pane.sorted_by()
            .iter()
            .position(|by| *by == "RUNS")
            .expect("offered")
            * 2
            + 2,
    );

    let rows = pane.rows(&reading, &Showing::default().sorted(most_first));
    let launches: Vec<&str> = rows
        .iter()
        .map(|row| row.key.as_str())
        .skip_while(|key| key.starts_with(ABOUT_THE_READING))
        .collect();

    assert_eq!(
        launches.first(),
        Some(&"run|alice|/usr/bin/nc.openbsd"),
        "{rows:?}"
    );
    assert!(
        rows.first()
            .is_some_and(|row| row.key.starts_with(ABOUT_THE_READING)),
        "what the reading could not finish is read before any order a reader chose: {rows:?}"
    );
    assert!(
        launches.iter().all(|key| key.starts_with("run|")),
        "no row about the reading is sorted in among the launches: {launches:?}"
    );
}

#[test]
fn sorted_by_program_the_rows_go_by_the_name_of_what_was_run_and_not_by_who_ran_it() {
    let reading = launches();
    let pane = pane();
    let by_name = Sorting::of(
        pane.sorted_by()
            .iter()
            .position(|by| *by == "PROGRAM")
            .expect("offered")
            * 2
            + 1,
    );

    let names: Vec<String> = pane
        .rows(&reading, &Showing::default().sorted(by_name))
        .into_iter()
        .filter(|row| row.key.starts_with("run|"))
        .map(|row| pane.cells(&reading, &row, Room::of(80))[1].text.clone())
        .collect();

    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names.first().map(String::as_str), Some("id"), "{names:?}");
}

#[test]
fn the_row_under_the_cursor_offers_its_person_and_its_program_to_narrow_the_list_to() {
    let reading = launches();

    let facets = pane().facets(
        &reading,
        &vigil_view::RowKey::of("run|alice|/usr/bin/nc.openbsd"),
    );

    assert_eq!(
        facets,
        vec![
            Facet::new("user", "alice"),
            Facet::new("program", "/usr/bin/nc.openbsd"),
        ]
    );
    assert!(
        pane()
            .facets(&reading, &vigil_view::RowKey::of("launches|capped"))
            .is_empty(),
        "a row about the reading has no person and no program, and narrowing to its \
         nothing would empty the list"
    );
}

#[test]
fn narrowed_to_a_person_the_list_holds_their_launches_and_the_footer_names_them() {
    let reading = launches();
    let pane = pane();
    let only = [Facet::new("user", "root")];
    let showing = Showing::default().narrowing(&only);

    let rows = pane.rows(&reading, &showing);
    let footer = pane.tally(&reading, &showing, rows.len());

    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(
        rows.iter().all(|row| row.key.starts_with("run|root|")),
        "{rows:?}"
    );
    assert!(footer.contains("2 of"), "{footer}");
    assert!(footer.contains("only user root"), "{footer}");
}

#[test]
fn narrowed_to_a_person_and_a_program_the_list_holds_what_both_of_them_name() {
    let reading = launches();
    let pane = pane();
    let only = [
        Facet::new("user", "root"),
        Facet::new("program", "/usr/bin/id"),
    ];

    let rows = pane.rows(&reading, &Showing::default().narrowing(&only));

    assert_eq!(
        rows.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(),
        vec!["run|root|/usr/bin/id"]
    );
}

#[test]
fn narrowed_to_nothing_the_list_says_how_to_get_the_rest_back() {
    let only = [Facet::new("program", "/usr/bin/nothing")];

    let notice = pane().empty(&Showing::default().narrowing(&only));

    assert!(notice.headline.contains("/usr/bin/nothing"), "{notice:?}");
    assert!(
        notice.detail.iter().any(|said| said.contains("everything")),
        "{notice:?}"
    );
}

#[test]
fn the_footer_says_that_this_list_only_grows() {
    let reading = launches();
    let pane = pane();

    let footer = pane.tally(&reading, &Showing::default(), 1);

    assert!(
        footer.contains("only grows"),
        "a row that stays after the program is gone is the point of this list: {footer}"
    );
}

#[test]
fn what_the_detail_of_a_launch_says_is_what_a_reader_can_act_on() {
    let reading = launches();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("run|"))
        .expect("the sample holds a launch");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("first seen"), "{said}");
    assert!(said.contains("runs"), "{said}");
    assert!(
        said.contains(&row.key),
        "the key an operator puts in suppressions is the finding key, and for this family it \
         is the row key whole: {said}"
    );
}
