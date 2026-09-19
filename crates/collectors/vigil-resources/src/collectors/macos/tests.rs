use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use serde_json::Value;
use vigil_collect::{Collector, Health, outside_the_sample};
use vigil_model::{Golden, Shape, Snapshot};

use super::ResourcesCollector;
use super::source::boot_time;
use crate::parsers::BOOT;

fn at_noon() -> String {
    "2026-09-19T12:00:00.000Z".to_string()
}

fn read() -> Snapshot {
    ResourcesCollector::new(at_noon)
        .collect()
        .expect("the kernel of a Mac answers every value this reads")
}

#[test]
fn the_boot_this_mac_is_running_and_the_moment_it_began_are_one_readable_row() {
    let snapshot = read();
    let boot = &snapshot.items[BOOT];

    assert_eq!(boot["readable"], true, "{boot}");
    assert_eq!(boot["boot_id"].as_str().map(str::len), Some(36), "{boot}");
    assert_eq!(boot["booted_at"].as_i64(), boot_time());
}

#[test]
fn the_memory_of_this_mac_is_read_and_its_swap_is_left_out_rather_than_written_as_zero() {
    let snapshot = read();
    let memory = &snapshot.items["memory|summary"];

    assert!(
        memory["total_bytes"]
            .as_u64()
            .is_some_and(|total| total > 0)
    );
    assert_eq!(memory["swap_total_bytes"], Value::Null);
}

#[test]
fn the_root_is_measured_as_the_read_only_system_it_is_on_the_disk_it_shares() {
    let snapshot = read();
    let root = snapshot
        .items
        .get("fs|/")
        .unwrap_or_else(|| panic!("{:?}", snapshot.items.keys().collect::<Vec<_>>()));

    assert_eq!(root["mount"], "/");
    assert_eq!(root["type"], "apfs");
    assert_eq!(root["read_only"], true);
    assert_eq!(root["storage_from"], "disk");
    assert!(
        root["storage"]
            .as_str()
            .and_then(|disk| disk.strip_prefix("disk"))
            .is_some_and(|number| number.chars().all(|digit| digit.is_ascii_digit())),
        "{root}"
    );
    assert!(root["free_percent_step"].is_u64());
}

#[test]
fn the_volume_this_mac_writes_to_is_measured_and_the_ones_the_system_keeps_are_not() {
    let snapshot = read();

    if let Some(data) = snapshot.items.get("fs|/System/Volumes/Data") {
        assert_eq!(data["read_only"], false);
        assert_eq!(
            data["storage"], snapshot.items["fs|/"]["storage"],
            "the system and its data are two volumes of one container, sharing its room"
        );
    }
    for kept in [
        "fs|/System/Volumes/VM",
        "fs|/System/Volumes/Preboot",
        "fs|/dev",
    ] {
        assert!(!snapshot.items.contains_key(kept), "{kept}");
    }
}

#[test]
fn a_reading_of_macos_has_the_shape_of_the_reading_the_rules_and_the_console_were_built_on() {
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("resources")
            .held()
            .expect("the published shape of resources"),
    )
    .expect("a shape");

    let drift = outside_the_sample(&Shape::of(&read()), &sample, &["memory.swap_total_bytes"]);

    assert!(drift.is_empty(), "{drift:#?}");
}

#[test]
fn a_mac_that_did_not_move_reads_the_same_twice() {
    let collector = ResourcesCollector::new(at_noon);

    let first = collector.collect().expect("read");
    let second = collector.collect().expect("read");

    assert_eq!(first.items, second.items);
}

#[test]
fn every_value_this_reads_is_one_an_unprivileged_account_is_shown() {
    assert_eq!(ResourcesCollector::new(at_noon).available(), Health::Ok);
}

#[test]
fn a_boot_time_that_moves_by_a_second_is_held_and_one_that_jumps_is_published() {
    let booted = Arc::new(AtomicI64::new(1_789_538_653));
    let source = Arc::clone(&booted);
    let collector =
        ResourcesCollector::with_boot_time(at_noon, move || Some(source.load(Ordering::SeqCst)));

    let first = collector.collect().expect("read");
    booted.store(1_789_538_654, Ordering::SeqCst);
    let second = collector.collect().expect("read");
    booted.store(1_789_542_253, Ordering::SeqCst);
    let third = collector.collect().expect("read");

    assert_eq!(first.items[BOOT]["booted_at"], 1_789_538_653);
    assert_eq!(second.items[BOOT]["booted_at"], 1_789_538_653);
    assert_eq!(
        third.items[BOOT]["booted_at"], 1_789_542_253,
        "macOS moves its boot time when the clock is set, and that move is what the rule reads"
    );
}

#[test]
fn a_restarted_agent_holds_the_boot_time_it_had_published_before() {
    let mut before = Snapshot::new("resources", at_noon());
    before.items.insert(
        BOOT.to_string(),
        serde_json::json!({"boot_id": null, "booted_at": 1_789_538_653, "readable": false}),
    );
    let collector = ResourcesCollector::with_boot_time(at_noon, || Some(1_789_538_654));

    collector.restore(&before);
    let read = collector.collect().expect("read");

    assert_eq!(read.items[BOOT]["booted_at"], 1_789_538_653);
}
