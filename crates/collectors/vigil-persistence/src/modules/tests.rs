use vigil_module::{Module, Settings};

use super::Persistence;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_what_starts_by_itself_walks_to_the_row_it_is_about() {
    assert_eq!(
        Persistence.row_of("persistence|cron|/etc/crontab|root|/x"),
        Some("cron|/etc/crontab|root|/x".to_string())
    );
    assert!(!Persistence.raised("port.listen|tcp|0.0.0.0:443"));
}

#[test]
fn a_subject_that_outlives_a_reboot_is_read_less_often_than_one_that_does_not() {
    assert_eq!(Persistence.name(), "persistence");
    assert_eq!(
        Persistence.every_seconds(),
        300,
        "a unit file waits; a socket exists between two readings or it never existed"
    );
    assert!(!Persistence.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Persistence.section().is_some());
}
