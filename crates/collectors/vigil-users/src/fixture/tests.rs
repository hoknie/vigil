use vigil_model::{Golden, Shape, Snapshot, class_of};

use super::users;

fn document(reading: &Snapshot) -> String {
    let mut text = serde_json::to_string_pretty(reading).expect("a reading is plain data");
    text.push('\n');
    text
}

#[test]
fn the_reading_on_disk_is_the_one_these_parsers_build_down_to_the_value() {
    if let Err(complaint) = Golden::reading("users").write_or_check(&document(&users())) {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_on_disk_is_the_shape_these_parsers_produce() {
    if let Err(complaint) = Golden::snapshot("users").write_or_check(&Shape::of(&users()).written())
    {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_published_to_the_console_is_the_shape_of_the_reading_published_beside_it() {
    let held = Golden::reading("users")
        .held()
        .expect("users has no reading on disk");
    let published: Snapshot = serde_json::from_str(&held).expect("a reading on disk parses");

    assert_eq!(
        Shape::of(&published).written(),
        Golden::snapshot("users")
            .held()
            .expect("users has no shape on disk")
    );
}

#[test]
fn a_reading_that_could_not_be_finished_says_so_in_a_row_of_its_own() {
    let reading = users();

    for marker in ["sshkey|backup|unreadable", "session-source|utmp"] {
        assert!(
            reading.items.contains_key(marker),
            "{marker} is a row the agent sends when it could not finish looking, and a sample \
             without it lets a screen that cannot draw it answer 'there is no attack'"
        );
    }
}

#[test]
fn every_class_the_parsers_build_is_named_in_the_sample_that_publishes_it() {
    let reading = users();
    let shape = Shape::of(&reading);

    for key in reading.items.keys() {
        assert!(
            shape.classes.contains_key(class_of(key)),
            "{key} builds no class in the shape of the reading"
        );
    }
}
