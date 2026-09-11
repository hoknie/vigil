use vigil_model::{Golden, Shape, Snapshot, class_of};

use crate::COLLECTORS;

use super::{accounts, launches, persistence, ports, processes};

fn readings() -> Vec<Snapshot> {
    vec![ports(), accounts(), processes(), persistence(), launches()]
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
