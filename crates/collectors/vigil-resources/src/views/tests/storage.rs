use vigil_view::conformance::{
    the_index_lists_every_search_and_sort_as_the_rows_do_in,
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in,
};
use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::scaled::many_filesystems;
use crate::fixture::resources;
use crate::views::TheHostAndItsFiles;

fn pane() -> Box<dyn Pane> {
    TheHostAndItsFiles.panes().remove(1)
}

fn opened(headings: &[&str]) -> Vec<Vec<String>> {
    let reading = resources();
    let pane = pane();
    let showing = Showing::default().opening(headings);

    pane.rows(&reading, &showing)
        .iter()
        .map(|row| {
            pane.cells(&reading, row, Room::of(80))
                .into_iter()
                .map(|cell| cell.text)
                .collect()
        })
        .collect()
}

#[test]
fn the_list_of_stores_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &resources());
}

#[test]
fn two_partitions_of_one_disk_and_a_volume_on_it_are_one_branch_and_not_three() {
    let closed = opened(&[]);

    let headings: Vec<&str> = closed.iter().map(|cells| cells[0].as_str()).collect();

    assert_eq!(
        headings,
        vec!["\u{25b8} sda (3)", "\u{25b8} sdb (1)", "\u{25b8} tmpfs (1)"],
        "/, /var and /home are written to one disk through a partition table and a logical \
         volume, and a reader who cannot see that reads three filesystems filling up where \
         there is one disk doing it"
    );
}

#[test]
fn opening_a_branch_lists_every_filesystem_written_to_that_store_and_how_full_each_one_is() {
    let rows = opened(&["storage|disk|sda"]);

    let mounts: Vec<&str> = rows.iter().map(|cells| cells[0].as_str()).collect();
    assert_eq!(
        mounts,
        vec![
            "\u{25be} sda (3)",
            "  /",
            "  /home",
            "  /var",
            "\u{25b8} sdb (1)",
            "\u{25b8} tmpfs (1)",
        ]
    );

    let under: Vec<&Vec<String>> = rows[1..4].iter().collect();
    for cells in under {
        assert!(
            cells.iter().any(|cell| cell.ends_with('%')),
            "a filesystem folded under its disk still has to say how full it is: {cells:?}"
        );
    }
}

#[test]
fn a_filesystem_on_no_block_device_is_gathered_under_what_mounted_it_rather_than_left_out() {
    let rows = opened(&["storage|source|tmpfs"]);

    let listed: Vec<&str> = rows.iter().map(|cells| cells[0].as_str()).collect();

    assert!(listed.contains(&"  /run"), "{listed:?}");
}

#[test]
fn a_store_holding_one_filesystem_says_so_and_holds_it_all_the_same() {
    let closed = opened(&[]);
    let sdb = closed
        .iter()
        .find(|cells| cells[0].contains("sdb"))
        .expect("the sample has a second disk");

    assert_eq!(sdb[0], "\u{25b8} sdb (1)");
    assert!(
        sdb.last().is_some_and(|size| size.ends_with("GB")),
        "{sdb:?}"
    );
}

#[test]
fn the_size_written_against_a_store_is_every_filesystem_on_it_added_up() {
    let reading = resources();
    let pane = pane();
    let rows = pane.rows(&reading, &Showing::default());
    let sda = rows
        .iter()
        .find(|row| row.key == "storage|disk|sda")
        .expect("the sample has one disk with three filesystems");

    let cells = pane.cells(&reading, sda, Room::of(80));
    let held: u64 = ["fs|/", "fs|/var", "fs|/home"]
        .iter()
        .map(|key| reading.items[*key]["total_bytes"].as_u64().unwrap_or(0))
        .sum();

    assert_eq!(
        cells.last().expect("a size column").text,
        vigil_view::bytes(held)
    );
}

#[test]
fn the_detail_of_a_store_names_what_it_was_read_from_and_warns_what_goes_with_it() {
    let reading = resources();
    let pane = pane();
    let row = vigil_view::RowKey::of("storage|disk|sda");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("partition table"), "{said}");
    assert!(said.contains("/var"), "{said}");
    assert!(said.contains("share one store"), "{said}");
}

#[test]
fn the_footer_counts_the_stores_as_well_as_the_filesystems_and_names_the_fullest() {
    let reading = resources();
    let pane = pane();
    let rows = pane.rows(&reading, &Showing::default());

    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(footer.contains("5 filesystem(s) on 3 store(s)"), "{footer}");
    assert!(footer.contains("least room on"), "{footer}");
}

#[test]
fn two_filesystems_of_one_size_under_two_headings_are_named_as_a_hint_and_never_as_a_grouping() {
    let mut reading = resources();
    let held = reading.items["fs|/var"]["total_bytes"].clone();
    if let Some(srv) = reading.items.get_mut("fs|/srv") {
        srv["total_bytes"] = held;
    }
    let pane = pane();

    let rows = pane.rows(&reading, &Showing::default());
    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(
        footer.contains("of the same size under different headings"),
        "{footer}"
    );
    assert_eq!(
        rows.iter().filter(|row| row.opens()).count(),
        3,
        "an equal size is a hint for a reader and never a reason to put two filesystems under \
         one heading: what this agent groups on is what the kernel says backs them"
    );
}

#[test]
fn every_search_and_every_opening_of_a_host_with_many_filesystems_lists_what_the_reading_lists() {
    let reading = many_filesystems(40);

    the_index_lists_every_search_and_sort_as_the_rows_do_in(
        pane().as_ref(),
        &reading,
        Showing::default(),
    );
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
        pane().as_ref(),
        &reading,
        Showing::default(),
    );
}
