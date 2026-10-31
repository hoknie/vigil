use vigil_model::Snapshot;
use vigil_view::{Pane, Room, Section, Showing, conformance};

use crate::fixture::network;
use crate::views::Listening;

fn pane(name: &str) -> Box<dyn Pane> {
    Listening
        .panes()
        .into_iter()
        .find(|pane| pane.name() == name)
        .unwrap_or_else(|| panic!("no list {name}"))
}

fn drawn(pane: &dyn Pane, reading: &Snapshot) -> Vec<Vec<String>> {
    pane.rows(reading, &Showing::default())
        .iter()
        .map(|row| {
            pane.cells(reading, row, Room::of(80))
                .into_iter()
                .map(|cell| cell.text)
                .collect()
        })
        .collect()
}

#[test]
fn the_pair_lists_answer_about_their_own_reading_and_answer_whole() {
    for name in ["outbound", "inbound"] {
        conformance::run_all(pane(name).as_ref(), &network());
    }
}

#[test]
fn the_outbound_list_names_the_program_and_the_peer() {
    let rows = drawn(pane("outbound").as_ref(), &network());

    assert!(
        rows.contains(&vec![
            "nc (deleted)".to_string(),
            "203.0.113.7:4444".to_string(),
            "tcp".to_string(),
            "www-data".to_string(),
        ]),
        "{rows:?}"
    );
    assert!(rows.iter().any(|row| row[0] == "curl"), "{rows:?}");
}

#[test]
fn the_inbound_list_names_the_program_the_port_and_the_peer() {
    let rows = drawn(pane("inbound").as_ref(), &network());

    assert_eq!(
        rows,
        vec![vec![
            "sshd".to_string(),
            "22".to_string(),
            "198.51.100.5".to_string(),
            "tcp".to_string(),
            "root".to_string(),
        ]]
    );
}

#[test]
fn the_outbound_footer_says_how_many_pairs_were_not_recorded() {
    let mut reading = network();
    reading.items.insert(
        "outbound|beyond".to_string(),
        serde_json::json!({"count": 388}),
    );

    let footer = pane("outbound").tally(&reading, &Showing::default(), 2);

    assert!(
        footer.starts_with("2 program-to-peer pair(s), read at "),
        "{footer}"
    );
    assert!(footer.ends_with(" · 388 more not recorded"), "{footer}");
}

#[test]
fn an_empty_outbound_list_says_this_host_talks_to_no_one() {
    let mut reading = network();
    reading
        .items
        .retain(|key, _| !key.starts_with("outbound|") && !key.starts_with("inbound|"));

    let outbound = pane("outbound");
    assert!(outbound.rows(&reading, &Showing::default()).is_empty());
    assert!(
        outbound
            .why_nothing_is_listed(&reading, &Showing::default())
            .is_none()
    );
    assert_eq!(
        outbound.empty(&Showing::default()).headline,
        "This host is not talking to anyone outside it."
    );
    assert_eq!(
        pane("inbound").empty(&Showing::default()).headline,
        "Nobody outside this host is talking to it."
    );
}

#[test]
fn switched_off_connections_are_said_and_not_shown_as_silence() {
    let mut reading = network();
    reading
        .items
        .retain(|key, _| !key.starts_with("outbound|") && !key.starts_with("inbound|"));
    reading.items.insert(
        crate::parsers::SWITCHED_OFF.to_string(),
        serde_json::json!({"read": false}),
    );

    for name in ["outbound", "inbound"] {
        let said = pane(name)
            .why_nothing_is_listed(&reading, &Showing::default())
            .expect("a reason");
        assert_eq!(
            said.headline,
            "Connections are not read here: connections is false in the network block."
        );
    }
}

#[test]
fn the_outbound_list_offers_nothing_to_close() {
    for name in ["outbound", "inbound"] {
        assert_eq!(
            pane(name).offers(),
            vigil_view::Offers::default(),
            "a pair has no pid, and closing it is no verb this console has"
        );
    }
}

#[test]
fn a_pair_is_not_listed_among_the_listening_sockets() {
    let rows = pane("sockets").rows(&network(), &Showing::default());

    assert!(
        rows.iter()
            .all(|row| !row.key.starts_with("outbound|") && !row.key.starts_with("inbound|")),
        "{rows:?}"
    );
}
