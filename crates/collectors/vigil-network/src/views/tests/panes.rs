use vigil_model::Snapshot;
use vigil_view::{Pane, Room, RowKey, Section, Showing, conformance};

use super::super::Listening;
use crate::fixture::{network, socket, socket_without_owner};

fn panes() -> Vec<Box<dyn Pane>> {
    Listening.panes()
}

#[test]
fn every_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    for pane in panes() {
        conformance::run_all(pane.as_ref(), &network());
    }
}

#[test]
fn a_socket_is_shown_as_the_reading_recorded_it_and_not_as_a_key_taken_apart() {
    let reading = network();
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
    assert_eq!(
        cells[3].text, "812",
        "the pid is the one value on this row an operator acts on, and a screen that shows \
         the program without it sends them to ps to find the same socket again"
    );
    assert!(cells[4].text.contains("sshd"), "{:?}", cells[4]);
}

#[test]
fn a_narrow_terminal_drops_the_command_and_never_a_column_the_reader_needs() {
    let pane = &panes()[0];

    let narrow = pane.columns(Room::of(80));
    let wide = pane.columns(Room::of(160));

    assert_eq!(narrow.len(), 5);
    assert_eq!(wide.len(), 6);
    assert_eq!(wide[5].header, "COMMAND");
    assert!(
        narrow.iter().any(|column| column.header == "PID"),
        "the command line is what an eighty-column terminal gives up; the pid is not, \
         because it is what the keys on this screen aim at"
    );
}

#[test]
fn the_kind_the_reader_switched_off_is_the_only_kind_missing_from_the_rows() {
    let reading = network();
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
    let reading = network();
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
fn grouping_by_program_opens_on_the_programs_alone_and_not_on_every_socket_of_each() {
    let reading = network();
    let pane = &panes()[1];

    let closed = pane.rows(&reading, &Showing::default());
    let heading = closed
        .iter()
        .find(|row| !row.of_the_reading)
        .expect("a program heading");

    assert!(heading.key.starts_with("program|") || heading.key == "unresolved");
    assert!(
        closed.iter().all(|row| row.depth == 0),
        "this screen is a tree, and a tree that arrives with every branch open is the flat \
         list with headings shuffled into it"
    );
    assert!(
        closed.iter().all(|row| row.gathers.is_some()),
        "every row of a closed tree is a heading, and each one carries how many sockets it \
         is holding back"
    );
    assert!(
        !pane.detail(&reading, heading, 80).is_empty(),
        "a heading says what it is a heading of"
    );
}

#[test]
fn a_program_the_reader_opened_shows_its_sockets_and_the_ones_beside_it_stay_shut() {
    let reading = network();
    let pane = &panes()[1];

    let closed = pane.rows(&reading, &Showing::default());
    let first = closed[0].key.clone();
    let opened = [first.as_str()];
    let rows = pane.rows(&reading, &Showing::default().opening(&opened));

    let under: Vec<&vigil_view::RowKey> = rows
        .iter()
        .filter(|row| row.gathered_under.as_deref() == Some(first.as_str()))
        .collect();

    assert_eq!(under.len(), closed[0].gathers.expect("a heading counts"));
    assert!(under.iter().all(|row| row.of_the_reading && row.depth == 1));
    assert_eq!(
        rows.iter().filter(|row| row.depth == 1).count(),
        under.len(),
        "opening one program must not open the next one: a reader who pressed the arrow once \
         has said one thing, not two"
    );
    assert!(
        rows.iter()
            .find(|row| row.key == first)
            .expect("still there")
            .opened,
        "the heading has to draw itself open, or the arrow that did it left no trace"
    );
}

#[test]
fn what_the_detail_of_a_socket_says_is_what_a_reader_can_act_on() {
    let reading = network();
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

fn two_programs_named_alike() -> Snapshot {
    let mut reading = Snapshot::new("network", "2026-09-14T09:00:00.000Z".to_string());
    for (key, item) in [
        (
            "tcp|0.0.0.0:80",
            socket("0.0.0.0", 80, "/usr/sbin/nginx", "www-data"),
        ),
        (
            "tcp|0.0.0.0:443",
            socket("0.0.0.0", 443, "/usr/sbin/nginx", "www-data"),
        ),
        (
            "tcp|0.0.0.0:8080",
            socket("0.0.0.0", 8080, "/opt/build/nginx", "deploy"),
        ),
        (
            "tcp|0.0.0.0:22",
            socket("0.0.0.0", 22, "/usr/sbin/sshd", "root"),
        ),
        ("tcp|0.0.0.0:9000", socket_without_owner("0.0.0.0", 9000)),
        (
            "tcp|127.0.0.1:9001",
            socket_without_owner("127.0.0.1", 9001),
        ),
    ] {
        reading.items.insert(key.to_string(), item);
    }
    reading
}

#[test]
fn a_heading_from_the_listing_draws_and_explains_itself_as_one_worked_out_from_the_reading() {
    let pane = &panes()[1];
    let opened = ["program|/usr/sbin/nginx", "unresolved"];

    for reading in [two_programs_named_alike(), network()] {
        let rows = pane.rows(&reading, &Showing::default().opening(&opened));
        let headings: Vec<&RowKey> = rows.iter().filter(|row| !row.of_the_reading).collect();

        assert!(headings.iter().any(|row| row.opened) && headings.iter().any(|row| !row.opened));
        for row in headings {
            assert!(
                row.gathers.is_some() && (row.key == "unresolved" || row.named.is_some()),
                "a heading from the listing that carries nothing makes this comparison one \
                 between the walk and itself: {row:?}"
            );
            let worked_out = RowKey::of(row.key.clone()).of_its_own().opened(row.opened);
            for room in [Room::of(80), Room::of(160)] {
                assert_eq!(
                    pane.cells(&reading, row, room),
                    pane.cells(&reading, &worked_out, room),
                    "what the listing knew about {} once is what every cell would have read \
                     the whole reading to learn",
                    row.key
                );
            }
            assert_eq!(
                pane.detail(&reading, row, 80),
                pane.detail(&reading, &worked_out, 80),
                "the detail of {} counts the sockets the heading above it counted",
                row.key
            );
        }
    }
}

#[test]
fn two_programs_sharing_a_name_are_each_shown_by_path_and_a_program_alone_by_its_name() {
    let reading = two_programs_named_alike();
    let pane = &panes()[1];

    let shown: Vec<String> = pane
        .rows(&reading, &Showing::default())
        .iter()
        .map(|row| pane.cells(&reading, row, Room::of(80))[0].text.clone())
        .collect();

    assert_eq!(
        shown,
        vec![
            "▸ /opt/build/nginx (1)",
            "▸ /usr/sbin/nginx (2)",
            "▸ sshd (1)",
            "▸ owner not resolved (2)",
        ],
        "two headings both reading nginx send the reader to open each one to learn which is \
         the copy nobody installed"
    );
}

#[test]
fn a_search_hiding_one_of_two_programs_sharing_a_name_leaves_the_other_shown_by_its_path() {
    let reading = two_programs_named_alike();
    let pane = &panes()[1];

    let rows = pane.rows(&reading, &Showing::searching("/opt/build"));
    let heading = rows
        .iter()
        .find(|row| row.key == "program|/opt/build/nginx")
        .expect("the search keeps the program it names");

    assert_eq!(
        pane.cells(&reading, heading, Room::of(80))[0].text,
        "▸ /opt/build/nginx (1)",
        "the whole path tells two programs on this host apart, and a search that hides one \
         of them has not taken it off the host"
    );
}
