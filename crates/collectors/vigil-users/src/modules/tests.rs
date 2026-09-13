use vigil_module::{Module, Settings};

use super::Users;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_an_account_a_group_or_a_key_walks_to_the_row_it_is_about() {
    for (key, row) in [
        ("user|account|backdoor", "account|backdoor"),
        ("user|group|docker", "group|docker"),
        ("user|sshkey|deploy|SHA256:abc", "sshkey|deploy|SHA256:abc"),
    ] {
        assert_eq!(Users.row_of(key), Some(row.to_string()), "{key}");
    }
    assert!(!Users.raised("port.listen|tcp|0.0.0.0:443"));
}

#[test]
fn the_rules_this_module_runs_are_the_ones_that_read_its_own_snapshot() {
    assert!(!Users.rules(&Settings::plain(at_noon)).is_empty());
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Users.name(), "users");
    assert_eq!(Users.every_seconds(), 300);
    assert_eq!(Users.unit(), None);
    assert!(Users.section().is_some(), "a subject a reader can open");
}
