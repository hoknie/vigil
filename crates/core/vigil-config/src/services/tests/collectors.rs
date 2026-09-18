use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::{blocks, blocks_in, collectors_path, seconds};

fn temporary(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-collectors-{name}-{}-{}",
        std::process::id(),
        NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("temp dir");
    directory
}

const CONTAINERS: &str = "\
containers:
  schedule: 60
---
containers-engines:
  schedule: 2m
  engines: [docker, podman]
";

#[test]
fn a_file_may_hold_several_documents_and_each_names_its_collectors() {
    let read = blocks_in(
        Path::new("/etc/vigil/collectors/containers.yaml"),
        CONTAINERS,
    )
    .expect("reads");

    let named: Vec<(&str, Option<u32>)> = read
        .iter()
        .map(|block| (block.name.as_str(), block.schedule))
        .collect();
    assert_eq!(
        named,
        [("containers", Some(60)), ("containers-engines", Some(120))]
    );
    assert!(
        read[0].settings.is_empty(),
        "the period is not a setting of the module"
    );
    assert_eq!(
        read[1].settings.get("engines"),
        Some(&serde_yaml::from_str::<Value>("[docker, podman]").expect("yaml"))
    );
}

#[test]
fn a_block_is_on_unless_it_says_it_is_off_and_says_so_with_a_word() {
    let read = blocks_in(
        Path::new("x.yaml"),
        "network:\nusers:\n  enabled: false\n  schedule: 300\n",
    )
    .expect("reads");

    assert!(
        read[0].enabled,
        "a block with nothing in it is a collector switched on"
    );
    assert!(!read[1].enabled);
    assert_eq!(
        read[1].schedule,
        Some(300),
        "and it keeps what it says while it is off"
    );
    assert!(blocks_in(Path::new("x.yaml"), "users:\n  enabled: no please\n").is_err());
}

#[test]
fn a_collector_written_in_two_files_is_refused_with_both_named() {
    let directory = temporary("twice");
    fs::write(directory.join("a.yaml"), "network:\n  schedule: 30\n").expect("writes");
    fs::write(directory.join("b.yaml"), "network:\n  schedule: 60\n").expect("writes");

    let refused = blocks(&directory).expect_err("must not be accepted");

    assert!(
        refused.contains("a.yaml") && refused.contains("b.yaml"),
        "{refused}"
    );
}

#[test]
fn every_file_of_the_directory_is_read_in_the_order_of_its_name() {
    let directory = temporary("order");
    fs::write(directory.join("users.yaml"), "users:\n").expect("writes");
    fs::write(directory.join("network.yml"), "network:\n").expect("writes");
    fs::write(directory.join("files.yaml.previous"), "files:\n").expect("writes");

    let names: Vec<String> = blocks(&directory)
        .expect("reads")
        .into_iter()
        .map(|block| block.name)
        .collect();

    assert_eq!(names, ["network", "users"]);
}

#[test]
fn a_period_is_seconds_or_a_number_with_its_unit() {
    for (written, expected) in [
        ("30", 30),
        ("30s", 30),
        ("5m", 300),
        ("1h", 3600),
        ("45 s", 45),
    ] {
        let value = serde_yaml::from_str::<Value>(written).expect("yaml");
        assert_eq!(seconds(&value), Ok(expected), "{written}");
    }
    for refused in ["0", "-5", "5 minutes", "1d", "soon", "[1]", "0s"] {
        let value = serde_yaml::from_str::<Value>(refused).expect("yaml");
        assert!(seconds(&value).is_err(), "{refused}");
    }
}

#[test]
fn a_block_that_is_not_a_mapping_is_refused_rather_than_read_as_empty() {
    for text in ["network: 30\n", "- network\n", "network: [a]\n"] {
        assert!(blocks_in(Path::new("x.yaml"), text).is_err(), "{text:?}");
    }
}

#[test]
fn the_configuration_points_at_the_collectors_the_same_way_it_points_at_suppressions() {
    assert_eq!(
        collectors_path(
            Path::new("/etc/vigil/vigil.yaml"),
            "collectors_path: collectors\n"
        ),
        Ok(Some(PathBuf::from("/etc/vigil/collectors")))
    );
    assert_eq!(
        collectors_path(Path::new("/etc/vigil/vigil.yaml"), "state_dir: /x\n"),
        Ok(None)
    );
}
