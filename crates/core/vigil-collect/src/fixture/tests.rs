use vigil_model::{Golden, Shape, Snapshot, class_of};

use crate::COLLECTORS;

use super::{
    accounts, containers, files, firewall, launches, persistence, ports, processes, resources,
};

fn readings() -> Vec<Snapshot> {
    vec![
        ports(),
        accounts(),
        processes(),
        persistence(),
        firewall(),
        launches(),
        resources(),
        containers(),
        files(),
    ]
}

fn document(reading: &Snapshot) -> String {
    let mut text = serde_json::to_string_pretty(reading).expect("a reading is plain data");
    text.push('\n');
    text
}

#[test]
fn the_reading_on_disk_is_the_one_these_parsers_build_down_to_the_value() {
    for reading in readings() {
        if let Err(complaint) = Golden::reading(&reading.source).write_or_check(&document(&reading))
        {
            panic!("{complaint}");
        }
    }
}

#[test]
fn the_shape_published_to_the_console_is_the_shape_of_the_reading_published_beside_it() {
    for reading in readings() {
        let held = Golden::reading(&reading.source)
            .held()
            .unwrap_or_else(|| panic!("{} has no reading on disk", reading.source));
        let published: Snapshot = serde_json::from_str(&held).expect("a reading on disk parses");

        assert_eq!(
            Shape::of(&published).written(),
            Golden::snapshot(&reading.source)
                .held()
                .unwrap_or_else(|| panic!("{} has no shape on disk", reading.source)),
            "the two samples of {} were written from two different readings, and the one the \
             console rehearses on is the one nobody checked",
            reading.source
        );
    }
}

#[test]
fn the_shape_on_disk_is_the_shape_these_parsers_produce() {
    for reading in readings() {
        let shape = Shape::of(&reading);
        if let Err(complaint) = Golden::snapshot(&reading.source).write_or_check(&shape.written()) {
            panic!("{complaint}");
        }
    }
}

#[test]
fn a_collector_in_the_vocabulary_without_a_published_shape_is_one_no_screen_has_seen() {
    let sampled: Vec<String> = readings()
        .iter()
        .map(|reading| reading.source.clone())
        .collect();

    for collector in COLLECTORS {
        assert!(
            sampled.iter().any(|source| source == collector.name),
            "{} is a collector this build ships and no sample says what it sends: a screen \
             drawing it is being tested against nothing",
            collector.name
        );
    }
}

#[test]
fn a_reading_that_could_not_be_finished_says_so_in_a_row_of_its_own() {
    let readings = readings();
    let all: Vec<&String> = readings
        .iter()
        .flat_map(|reading| reading.items.keys())
        .collect();

    for marker in [
        "processes|unresolved",
        "launches|capped",
        "launches|unnamed",
        "launches|dropping",
        "launches|source",
        "modules|unreadable",
        "unix|unnamed",
        "sshkey|backup|unreadable",
        "session-source|utmp",
    ] {
        assert!(
            all.iter().any(|key| *key == marker),
            "{marker} is a row the agent sends when it could not finish looking, and no sample \
             carries it: a screen that cannot draw it answers 'there is no attack'"
        );
    }
}

#[test]
fn every_class_the_parsers_build_is_named_in_the_sample_that_publishes_it() {
    for reading in readings() {
        let shape = Shape::of(&reading);
        for key in reading.items.keys() {
            assert!(
                shape.classes.contains_key(class_of(key)),
                "{key} builds no class in the shape of {}",
                reading.source
            );
        }
    }
}
