use serde_json::json;
use vigil_module::{Module, Settings};

use super::Resources;
use super::resources::Thresholds;
use crate::rules::{CLOCK_SKEW_SECONDS, DISK_FREE_PERCENT, INODE_FREE_PERCENT};

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_this_host_walks_to_the_row_of_the_reading_it_is_about() {
    for (key, row) in [
        ("resource|boot", "boot|current"),
        ("resource|clock", "boot|current"),
        ("resource|disk|/var", "fs|/var"),
        ("resource|inodes|/var", "fs|/var"),
    ] {
        assert_eq!(Resources.row_of(key), Some(row.to_string()), "{key}");
    }
    assert!(!Resources.raised("file|/etc/hosts"));
    assert_eq!(Resources.row_of("file|/etc/hosts"), None);
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Resources.name(), "resources");
    assert_eq!(Resources.every_seconds(), 60);
    assert_eq!(Resources.unit(), None);
    assert!(!Resources.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Resources.section().is_some());
}

#[test]
fn a_host_whose_file_names_a_limit_is_watched_by_that_one_and_not_by_the_shipped_one() {
    let named = Settings::of(
        at_noon,
        "resources",
        serde_json::json!({ "disk_free_percent": 25 }),
    );
    let thresholds: Thresholds = named.read().expect("the sample parses");

    assert_eq!(thresholds.limits().disk_free_percent, 25);
    assert_eq!(
        thresholds.limits().clock_skew_seconds,
        300,
        "a file that names one limit leaves the others on the ones this module ships"
    );
}

#[test]
fn a_limit_that_would_report_every_reading_is_refused_at_the_door_and_not_at_the_first_tick() {
    for (said, complained_about) in [
        (json!({"clock_skew_seconds": 0}), "clock_skew_seconds"),
        (json!({"disk_free_percent": 101}), "disk_free_percent"),
        (json!({"inode_free_percent": 101}), "inode_free_percent"),
    ] {
        let refusal = Resources
            .check(&Settings::of(at_noon, "resources", said.clone()))
            .expect_err("a limit no reading can pass is a limit nobody meant to write");

        assert!(refusal.contains(complained_about), "{said}: {refusal}");
    }
}

#[test]
fn a_key_this_module_does_not_know_is_refused_rather_than_quietly_left_out() {
    let refusal = Resources
        .check(&Settings::of(
            at_noon,
            "resources",
            json!({"disk_free_percnt": 25}),
        ))
        .expect_err("a misspelled limit reads as the shipped one, and nothing says so");

    assert!(refusal.contains("disk_free_percnt"), "{refusal}");
}

#[test]
fn the_limits_a_file_says_nothing_about_are_the_ones_the_rules_of_this_module_declare() {
    let quiet = Settings::plain(at_noon);

    assert!(Resources.check(&quiet).is_ok());
    assert_eq!(
        Thresholds::default().limits(),
        vigil_rules_limits(),
        "the daemon holds no copy of these numbers any more: the module is where they are \
         written and the only place a reader has to look"
    );
}

fn vigil_rules_limits() -> crate::rules::ResourceLimits {
    crate::rules::ResourceLimits {
        clock_skew_seconds: i64::from(CLOCK_SKEW_SECONDS),
        disk_free_percent: u64::from(DISK_FREE_PERCENT),
        inode_free_percent: u64::from(INODE_FREE_PERCENT),
    }
}
