use vigil_view::{Pane, Room, Section, Showing, Sorting, conformance};

use super::TheHostAndItsFiles;
use crate::fixture::files;

fn pane() -> Box<dyn Pane> {
    TheHostAndItsFiles.panes().remove(0)
}

fn drawn(room: u16) -> Vec<Vec<String>> {
    let reading = files();
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
    conformance::run_all(pane().as_ref(), &files());
}

#[test]
fn the_files_an_operator_named_are_read_before_the_directories_this_agent_adds() {
    let reading = files();
    let rows = pane().rows(&reading, &Showing::default());

    let kinds: Vec<&str> = rows
        .iter()
        .map(|row| row.key.split('|').next().unwrap_or(""))
        .collect();

    let last_file = kinds.iter().rposition(|kind| *kind == "file");
    let first_directory = kinds.iter().position(|kind| *kind == "directory");

    assert!(last_file < first_directory, "{kinds:?}");
}

#[test]
fn a_path_that_is_not_on_this_host_says_so_rather_than_reading_as_an_ordinary_file() {
    let missing = drawn(160)
        .into_iter()
        .find(|cells| cells[1] == "/etc/pam.d/sshd")
        .expect("the sample names a path that is not there");

    assert_eq!(missing.last().map(String::as_str), Some("not there"));
}

#[test]
fn a_narrow_terminal_drops_the_hash_and_the_size_and_keeps_the_mode_and_the_owner() {
    let wide = pane().columns(Room::of(160));
    let narrow = pane().columns(Room::of(80));

    let named = |columns: &[vigil_view::Column]| -> Vec<&'static str> {
        columns.iter().map(|column| column.header).collect()
    };

    assert_eq!(
        named(&narrow),
        vec!["KIND", "PATH", "MODE", "OWNER", "STANDING"]
    );
    assert!(named(&wide).contains(&"SHA256"));
    assert_eq!(drawn(80)[0].len(), narrow.len());
}

#[test]
fn the_footer_counts_the_files_the_directories_and_the_paths_that_are_not_there() {
    let reading = files();
    let pane = pane();
    let rows = pane.rows(&reading, &Showing::default());

    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(footer.contains("file(s)"), "{footer}");
    assert!(footer.contains("directory(s)"), "{footer}");
    assert!(footer.contains("1 not on this host"), "{footer}");
}

#[test]
fn the_reader_may_put_the_watched_paths_in_the_order_of_their_mode() {
    let reading = files();
    let pane = pane();
    let by_mode = Showing::default().sorted(Sorting {
        by: 3,
        descending: false,
    });

    let modes: Vec<String> = pane
        .rows(&reading, &by_mode)
        .iter()
        .map(|row| {
            reading.items[&row.key]["mode"]
                .as_str()
                .unwrap_or("—")
                .to_string()
        })
        .collect();

    let mut sorted = modes.clone();
    sorted.sort();

    assert_eq!(modes, sorted, "{modes:?}");
}

#[test]
fn the_detail_of_a_watched_file_holds_the_hash_that_is_what_says_its_content_changed() {
    let reading = files();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == "file|/etc/ssh/sshd_config")
        .expect("the sample watches sshd_config");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("sha256"), "{said}");
    assert!(said.contains(&row.key), "{said}");
}

#[test]
fn a_search_that_matches_nothing_says_what_was_searched_for_rather_than_drawing_an_empty_table() {
    let reading = files();
    let pane = pane();
    let searching = Showing::searching("nothing-of-the-sort");

    assert!(pane.rows(&reading, &searching).is_empty());
    assert!(
        pane.empty(&searching)
            .headline
            .contains("nothing-of-the-sort")
    );
}
