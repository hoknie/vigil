use vigil_module::{Module, Settings};

use super::Network;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-12T12:00:00.000Z".to_string()
}

#[test]
fn the_findings_of_this_module_walk_to_the_row_of_the_socket_they_are_about() {
    assert_eq!(
        Network.row_of("port.listen|tcp|0.0.0.0:4444"),
        Some("tcp|0.0.0.0:4444".to_string()),
        "the console finds the row by the key the collector wrote, and asks this module \
         rather than a table of its own"
    );
    assert!(!Network.raised("user|account|backdoor"));
}

#[test]
fn the_rules_this_module_runs_are_the_ones_that_read_its_own_snapshot() {
    let rules = Network.rules(&Settings::plain(at_noon));

    assert!(
        !rules.is_empty(),
        "a module that watches a host and judges nothing is a reading nobody sees"
    );
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Network.name(), "network");
    assert_eq!(Network.every_seconds(), 30);
    assert_eq!(
        Network.unit(),
        None,
        "nothing on the host has to be started for /proc/net"
    );
    assert!(!Network.subject().is_empty());
    assert!(Network.section().is_some());
}
