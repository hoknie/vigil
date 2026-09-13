use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::Listening;
use crate::fixture::ports;

fn panes() -> Vec<Box<dyn Pane>> {
    Listening.panes()
}

#[test]
fn every_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    for pane in panes() {
        conformance::run_all(pane.as_ref(), &ports());
    }
}

#[test]
fn a_socket_is_shown_as_the_reading_recorded_it_and_not_as_a_key_taken_apart() {
    let reading = ports();
    let pane = &panes()[0];
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == "tcp|0.0.0.0:22")
        .expect("the sample listens on 22");

    let cells = pane.cells(&reading, &row, Room::of(160));

    assert_eq!(cells[0].text, "tcp");
    assert_eq!(cells[1].text, "0.0.0.0:22");
    assert_eq!(cells[2].text, "root");
    assert!(cells[3].text.contains("sshd"), "{:?}", cells[3]);
}

#[test]
fn a_narrow_terminal_drops_the_command_and_never_a_column_the_reader_needs() {
    let pane = &panes()[0];

    let narrow = pane.columns(Room::of(80));
    let wide = pane.columns(Room::of(160));

    assert_eq!(narrow.len(), 4);
    assert_eq!(wide.len(), 5);
    assert_eq!(wide[4].header, "COMMAND");
}

#[test]
fn the_kind_the_reader_switched_off_is_the_only_kind_missing_from_the_rows() {
    let reading = ports();
    let pane = &panes()[0];

    let whole = pane.rows(&reading, &Showing::default());
    let without_unix = pane.rows(&reading, &Showing::default().hiding(&["unix"]));

    assert!(whole.len() > without_unix.len());
    assert!(
        without_unix.iter().all(|row| !row.key.starts_with("unix|")),
        "a kind switched off here is hidden here; the agent read it either way"
    );
}

#[test]
fn a_word_the_reader_typed_narrows_by_everything_recorded_about_a_socket() {
    let reading = ports();
    let pane = &panes()[0];

    let matching = pane.rows(&reading, &Showing::searching("sshd"));

    assert!(!matching.is_empty());
    assert!(
        matching.iter().all(|row| {
            let item = &reading.items[&row.key];
            vigil_view::haystack(&row.key, item).contains("sshd")
        }),
        "the search reads the whole row, not its key"
    );
}

#[test]
fn grouping_by_program_puts_a_heading_of_its_own_over_the_sockets_of_one_program() {
    let reading = ports();
    let pane = &panes()[1];

    let rows = pane.rows(&reading, &Showing::default());
    let heading = rows
        .iter()
        .find(|row| !row.of_the_reading)
        .expect("a program heading");

    assert!(heading.key.starts_with("program|") || heading.key == "unresolved");
    assert!(
        rows.iter().any(|row| row.depth == 1),
        "the sockets of a program are drawn under it"
    );
    assert!(
        !pane.detail(&reading, heading, 80).is_empty(),
        "a heading says what it is a heading of"
    );
}

#[test]
fn what_the_detail_of_a_socket_says_is_what_a_reader_can_act_on() {
    let reading = ports();
    let pane = &panes()[0];
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == "tcp|0.0.0.0:22")
        .expect("the sample listens on 22");

    let pieces = pane.detail(&reading, &row, 80);
    let said = format!("{pieces:?}");

    assert!(said.contains("0.0.0.0:22"), "{said}");
    assert!(
        said.contains("port.listen|tcp|0.0.0.0:22"),
        "the key an operator puts in suppressions is the finding key, not the row key: {said}"
    );
}
