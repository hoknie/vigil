use vigil_model::{Golden, Shape, Snapshot, class_of};

use super::firewall;

fn document(reading: &Snapshot) -> String {
    let mut text = serde_json::to_string_pretty(reading).expect("a reading is plain data");
    text.push('\n');
    text
}

#[test]
fn the_reading_on_disk_is_the_one_these_parsers_build_down_to_the_value() {
    if let Err(complaint) = Golden::reading("firewall").write_or_check(&document(&firewall())) {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_on_disk_is_the_shape_these_parsers_produce() {
    if let Err(complaint) =
        Golden::snapshot("firewall").write_or_check(&Shape::of(&firewall()).written())
    {
        panic!("{complaint}");
    }
}

#[test]
fn the_shape_published_to_the_console_is_the_shape_of_the_reading_published_beside_it() {
    let held = Golden::reading("firewall")
        .held()
        .expect("firewall has no reading on disk");
    let published: Snapshot = serde_json::from_str(&held).expect("a reading on disk parses");

    assert_eq!(
        Shape::of(&published).written(),
        Golden::snapshot("firewall")
            .held()
            .expect("firewall has no shape on disk")
    );
}

#[test]
fn every_class_the_parsers_build_is_named_in_the_sample_that_publishes_it() {
    let reading = firewall();
    let shape = Shape::of(&reading);

    for key in reading.items.keys() {
        assert!(
            shape.classes.contains_key(class_of(key)),
            "{key} builds no class in the shape of the reading"
        );
    }
}
