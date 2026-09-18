use vigil_model::{Golden, Shape, Snapshot, class_of};

use super::network;

fn document(reading: &Snapshot) -> String {
    let mut text = serde_json::to_string_pretty(reading).expect("a reading is plain data");
    text.push('\n');
    text
}

#[test]
fn the_reading_on_disk_is_the_one_these_parsers_build_down_to_the_value() {
    if let Err(complaint) = Golden::reading("network").write_or_check(&document(&network())) {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_on_disk_is_the_shape_these_parsers_produce() {
    if let Err(complaint) =
        Golden::snapshot("network").write_or_check(&Shape::of(&network()).written())
    {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_published_to_the_console_is_the_shape_of_the_reading_published_beside_it() {
    let held = Golden::reading("network")
        .held()
        .expect("network has no reading on disk");
    let published: Snapshot = serde_json::from_str(&held).expect("a reading on disk parses");

    assert_eq!(
        Shape::of(&published).written(),
        Golden::snapshot("network")
            .held()
            .expect("network has no shape on disk"),
        "the two samples were written from two different readings, and the one the console \
         rehearses on is the one nobody checked"
    );
}

#[test]
fn a_reading_that_could_not_be_finished_says_so_in_a_row_of_its_own() {
    assert!(
        network().items.contains_key("unix|unnamed"),
        "a listening unix socket whose name is gone is counted in a row of its own, and a \
         sample without that row lets a screen that cannot draw it answer 'there is no attack'"
    );
}

#[test]
fn every_class_the_parsers_build_is_named_in_the_sample_that_publishes_it() {
    let reading = network();
    let shape = Shape::of(&reading);

    for key in reading.items.keys() {
        assert!(
            shape.classes.contains_key(class_of(key)),
            "{key} builds no class in the shape of the reading"
        );
    }
}
