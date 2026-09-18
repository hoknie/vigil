use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::conformance::the_index_lists_every_search_and_sort_as_the_rows_do_in;
use vigil_view::{Assembled, Pane, Section, Showing, listed, listing};

use super::super::Listening;
use crate::fixture::network;

pub(super) const KINDS_SWITCHED_OFF: [&[&str]; 5] = [
    &["unix"],
    &["tcp", "udp6"],
    &["tcp6", "udp"],
    &["tcp", "tcp6", "udp", "udp6", "unix"],
    &["not a kind of socket"],
];

fn panes() -> Vec<Box<dyn Pane>> {
    Listening.panes()
}

pub(super) fn scaled(copies: usize) -> Snapshot {
    let sample = network();
    let mut reading = sample.clone();
    reading.items.clear();
    for copy in 0..copies {
        for (key, item) in &sample.items {
            let mut item = item.clone();
            if copy % 4 == 3 {
                moved(&mut item, copy);
            }
            reading.items.insert(format!("{key}{copy:03}"), item);
        }
    }
    reading
}

fn moved(item: &mut Value, copy: usize) {
    let path = item
        .pointer("/process/exe")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(path) = path {
        item["process"]["exe"] = json!(format!("/opt/release-{}{path}", copy % 8));
    }
}

fn indexed(pane: &dyn Pane, reading: &Snapshot, showing: &Showing<'_>) -> usize {
    pane.index(reading, showing)
        .expect("every pane of this crate answers from an index")
        .len()
}

#[test]
fn the_socket_list_with_kinds_switched_off_lists_every_search_and_sort_as_its_rows_do() {
    let pane = &panes()[0];

    for reading in [network(), scaled(6)] {
        for hidden in KINDS_SWITCHED_OFF {
            let showing = Showing::default().hiding(hidden);
            the_index_lists_every_search_and_sort_as_the_rows_do_in(
                pane.as_ref(),
                &reading,
                showing,
            );

            assert_eq!(
                indexed(pane.as_ref(), &reading, &showing),
                pane.rows(&reading, &showing).len(),
                "the index is built once for the kinds the reader left on, so it holds every \
                 socket of those kinds and not one socket of a kind switched off: {hidden:?}"
            );
        }
    }
}

#[test]
fn the_tree_opened_at_any_heading_lists_every_search_as_its_rows_do() {
    let pane = &panes()[1];
    let reading = scaled(4);
    let headings: Vec<String> = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .map(|row| row.key)
        .collect();
    let every: Vec<&str> = headings.iter().map(String::as_str).collect();

    assert!(
        every.len() > 6 && every.contains(&"unresolved"),
        "the reading this test opens has to hold several programs and sockets with no owner, \
         or opening them proves nothing: {every:?}"
    );
    let openings: [&[&str]; 5] = [
        &every[..1],
        &["unresolved"],
        &[every[1], every[every.len() - 2]],
        &every,
        &["program|/nowhere"],
    ];
    for opened in openings {
        for hidden in [&[][..], KINDS_SWITCHED_OFF[0], KINDS_SWITCHED_OFF[1]] {
            the_index_lists_every_search_and_sort_as_the_rows_do_in(
                pane.as_ref(),
                &reading,
                Showing::default().opening(opened).hiding(hidden),
            );
        }
    }
}

#[test]
fn a_larger_reading_with_ties_and_programs_sharing_a_name_lists_from_the_index_as_it_is_read() {
    let reading = scaled(24);
    let opened = [
        "program|/usr/sbin/nginx",
        "program|/opt/release-3/usr/sbin/nginx",
    ];

    for pane in panes() {
        let showing = Showing::default().opening(&opened);
        the_index_lists_every_search_and_sort_as_the_rows_do_in(pane.as_ref(), &reading, showing);
        assert!(
            indexed(pane.as_ref(), &reading, &showing) > 250,
            "a larger reading is what puts many rows under one user, one program and one pid"
        );
    }

    let tree = &panes()[1];
    let index = tree
        .index(&reading, &Showing::default().opening(&opened))
        .expect("the tree answers from an index");
    for search in [
        "release-3",
        "/usr/sbin/nginx",
        "NGINX",
        "dockerd",
        "permission",
    ] {
        let showing = Showing::searching(search).opening(&opened);
        let (_, rows) = listed(tree.as_ref(), &reading, &showing, &index, None);

        assert_eq!(
            rows,
            tree.rows(&reading, &showing),
            "the tree lists from its index what it lists from the reading for {search:?}"
        );
        assert!(
            rows.iter()
                .filter(|row| !row.of_the_reading && row.key != "unresolved")
                .all(|row| row
                    .named
                    .as_deref()
                    .is_some_and(|name| name.starts_with('/'))),
            "every program in this reading shares its name with a copy under /opt, and a search \
             that leaves one of the two on screen has not taken the other off the host: \
             {search:?} {rows:?}"
        );
    }
}

#[test]
fn the_tree_keeps_the_numbers_of_its_sockets_and_copies_no_row_of_the_reading() {
    let tree = &panes()[1];
    let reading = scaled(4);
    let every: Vec<String> = tree
        .rows(&reading, &Showing::default())
        .into_iter()
        .map(|row| row.key)
        .collect();
    let opened: Vec<&str> = every.iter().map(String::as_str).collect();
    let showing = Showing::default().opening(&opened);
    let index = tree
        .index(&reading, &showing)
        .expect("the tree answers from an index");

    let (_, assembled) = listing(tree.as_ref(), &reading, &showing, &index, None);

    assert!(
        matches!(assembled, Assembled::Gathered(_)),
        "the console lets go of the last list on every keystroke, and a tree of row numbers is \
         let go of at once where copies of its sockets are freed one by one"
    );
}

#[test]
fn every_socket_is_indexed_under_the_number_of_its_program_and_the_unresolved_come_last() {
    let tree = &panes()[1];
    let reading = scaled(4);
    let index = tree
        .index(&reading, &Showing::default())
        .expect("the tree answers from an index");

    let mut numbered: Vec<(Option<usize>, &str)> = (0..index.len())
        .map(|at| {
            (
                index.heading_of(at),
                index.row(at).gathered_under.as_deref().unwrap_or(""),
            )
        })
        .collect();
    numbered.sort_unstable();
    numbered.dedup();

    assert!(
        numbered.iter().all(|(number, _)| number.is_some()),
        "the tree lays its sockets out by the number of their program, and a socket without one \
         would be laid out under no heading: {numbered:?}"
    );
    assert!(
        numbered.windows(2).all(|pair| pair[0].0 != pair[1].0),
        "one number stands for one heading, or two programs would be folded into one: \
         {numbered:?}"
    );
    let headings: Vec<&str> = numbered.iter().map(|(_, heading)| *heading).collect();
    assert_eq!(
        headings.last(),
        Some(&"unresolved"),
        "sockets with no owner are listed after every program"
    );
    assert!(
        headings[..headings.len() - 1]
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
        "the numbers follow the headings in order, so laying the tree out by number lists the \
         programs in the order they are read: {headings:?}"
    );
}

#[test]
fn sockets_of_one_program_share_one_heading_and_one_name_and_do_not_carry_their_own() {
    let tree = &panes()[1];
    let reading = scaled(4);
    let index = tree
        .index(&reading, &Showing::default())
        .expect("the tree answers from an index");

    let mut compared = 0;
    for at in 1..index.len() {
        let (before, here) = (index.row(at - 1), index.row(at));
        if index.heading_of(at - 1) != index.heading_of(at) {
            continue;
        }
        let (Some(was), Some(is)) = (&before.gathered_under, &here.gathered_under) else {
            continue;
        };
        compared += 1;
        assert!(
            std::sync::Arc::ptr_eq(was, is),
            "{} and {} stand under one program, and the index writes that heading once for the \
             program and not once for every socket",
            before.key,
            here.key
        );
        if let (Some(was), Some(is)) = (&before.named, &here.named) {
            assert!(
                std::sync::Arc::ptr_eq(was, is),
                "the name of a program is worked out once for the program and shared by its sockets"
            );
        }
    }
    assert!(
        compared > 0,
        "the reading has to put two sockets of one program next to each other, or nothing is compared"
    );
}
