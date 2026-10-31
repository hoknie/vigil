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

#[test]
fn a_network_block_without_the_key_reads_connections() {
    let reading: super::network::Reading = vigil_module::Settings::of(
        || "2026-10-31T12:00:00.000Z".to_string(),
        "network",
        serde_json::json!({}),
    )
    .read()
    .expect("an empty block parses");

    assert!(reading.connections);
}

#[test]
fn a_connections_key_that_is_not_true_or_false_stops_the_daemon_by_name() {
    use vigil_module::Module;

    for said in [
        serde_json::json!({"connections": "yes"}),
        serde_json::json!({"connectoins": false}),
    ] {
        let refusal = super::Network
            .check(&vigil_module::Settings::of(
                || "2026-10-31T12:00:00.000Z".to_string(),
                "network",
                said.clone(),
            ))
            .expect_err("refused at the door");

        assert!(refusal.contains("connect"), "{said}: {refusal}");
    }
}

#[test]
fn connections_switched_off_leave_only_the_listening_sockets() {
    let snapshot = crate::parsers::listening_snapshot(
        "2026-10-31T12:00:00.000Z",
        &crate::parsers::SocketsReading {
            network: &[],
            unix: &[],
            unnamed_unix: 0,
            owners: &std::collections::BTreeMap::new(),
            users: &std::collections::BTreeMap::new(),
            pairs: None,
        },
    );

    assert!(
        !snapshot
            .items
            .keys()
            .any(|key| key.starts_with("outbound|") || key.starts_with("inbound|"))
    );
    assert!(
        snapshot.items.contains_key(crate::parsers::SWITCHED_OFF),
        "the screen says the pairs are switched off rather than that nobody talks"
    );
}

#[test]
fn a_finding_about_a_conversation_walks_to_its_row_in_the_outbound_list() {
    assert_eq!(
        Network.row_of("connection.outbound|tcp|203.0.113.7:4444|/tmp/.x/nc"),
        Some("outbound|tcp|203.0.113.7:4444|/tmp/.x/nc".to_string())
    );
    assert!(Network.raised("connection.outbound|tcp|203.0.113.7:4444|/tmp/.x/nc"));
}
