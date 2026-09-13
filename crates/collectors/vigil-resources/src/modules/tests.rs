use vigil_module::{Module, Settings};

use super::Resources;
use super::resources::Thresholds;

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
