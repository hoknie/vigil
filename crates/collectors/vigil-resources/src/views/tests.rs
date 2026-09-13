use vigil_view::{Pane, Room, Section, Showing, Sorting, conformance};

use super::TheHostAndItsFiles;
use crate::fixture::resources;

fn pane() -> Box<dyn Pane> {
    TheHostAndItsFiles.panes().remove(0)
}

fn drawn(room: u16) -> Vec<Vec<String>> {
    let reading = resources();
    let pane = pane();

    pane.rows(&reading, &Showing::default())
        .iter()
        .map(|row| {
            pane.cells(&reading, row, Room::of(room))
                .into_iter()
                .map(|cell| cell.text)
                .collect()
        })
        .collect()
}

#[test]
fn the_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &resources());
}

#[test]
fn the_boot_and_the_memory_are_read_before_the_filesystems_this_host_holds_its_files_on() {
    let reading = resources();
    let rows = pane().rows(&reading, &Showing::default());

    let kinds: Vec<&str> = rows
        .iter()
        .map(|row| row.key.split('|').next().unwrap_or(""))
        .collect();

    assert_eq!(kinds.first(), Some(&"boot"), "{kinds:?}");
    assert_eq!(kinds.get(1), Some(&"memory"), "{kinds:?}");
    assert!(kinds[2..].iter().all(|kind| *kind == "fs"), "{kinds:?}");
}

#[test]
fn a_filesystem_says_how_much_room_is_left_on_it_as_a_step_and_never_as_the_bytes() {
    let rows = drawn(160);
    let filesystem = rows
        .iter()
        .find(|cells| cells[1] == "/var")
        .expect("the sample mounts /var");

    assert_eq!(filesystem[0], "filesystem");
    assert!(
        filesystem.iter().any(|cell| cell.ends_with('%')),
        "{filesystem:?}"
    );
    assert!(
        filesystem.iter().any(|cell| cell == "read write"),
        "{filesystem:?}"
    );
}

#[test]
fn a_narrow_terminal_drops_the_columns_about_the_device_and_keeps_the_room_left() {
    let wide = pane().columns(Room::of(160));
    let narrow = pane().columns(Room::of(80));

    let named = |columns: &[vigil_view::Column]| -> Vec<&'static str> {
        columns.iter().map(|column| column.header).collect()
    };

    assert_eq!(
        named(&narrow),
        vec!["KIND", "WHAT", "FREE", "INODES", "SIZE"]
    );
    assert!(named(&wide).contains(&"DEVICE"));
    assert!(named(&wide).contains(&"MOUNTED"));
    assert_eq!(drawn(80)[0].len(), narrow.len());
}

#[test]
fn the_footer_names_the_filesystem_with_the_least_room_left_on_it() {
    let reading = resources();
    let pane = pane();
    let rows = pane.rows(&reading, &Showing::default());

    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(footer.contains("least room on"), "{footer}");
    assert!(footer.contains("filesystem(s)"), "{footer}");
}

#[test]
fn the_reader_may_put_the_filesystems_in_the_order_of_the_room_left_on_them() {
    let reading = resources();
    let pane = pane();
    let by_free = Showing::default().sorted(Sorting {
        by: 3,
        descending: false,
    });

    let rows = pane.rows(&reading, &by_free);
    let free: Vec<Option<u64>> = rows
        .iter()
        .map(|row| reading.items[&row.key]["free_percent_step"].as_u64())
        .collect();

    let mut sorted = free.clone();
    sorted.sort_by_key(|step| match step {
        Some(step) => format!("{step:0>3}"),
        None => String::new(),
    });

    assert_eq!(free, sorted, "{free:?}");
}

#[test]
fn the_detail_of_a_row_holds_every_field_the_agent_wrote_down_about_it() {
    let reading = resources();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("fs|"))
        .expect("the sample mounts a filesystem");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("mount"), "{said}");
    assert!(said.contains("free percent step"), "{said}");
    assert!(said.contains(&row.key), "{said}");
}

#[test]
fn a_search_that_matches_nothing_says_what_was_searched_for_rather_than_drawing_an_empty_table() {
    let reading = resources();
    let pane = pane();
    let searching = Showing::searching("nothing-of-the-sort");

    assert!(pane.rows(&reading, &searching).is_empty());
    assert!(
        pane.empty(&searching)
            .headline
            .contains("nothing-of-the-sort")
    );
}
