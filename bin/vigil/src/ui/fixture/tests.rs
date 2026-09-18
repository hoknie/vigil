use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::{CollectorStatus, Golden, Settled, Shape, Snapshot};

#[test]
fn every_reading_the_console_draws_is_the_one_the_module_that_took_it_publishes() {
    for (collector, drawn) in [
        ("resources", vigil_resources::fixture::resources()),
        ("files", vigil_files::fixture::files()),
        ("ports", vigil_network::fixture::ports()),
        ("containers", vigil_containers::fixture::containers()),
        ("containers-engines", vigil_engines::fixture::engines()),
    ] {
        assert_eq!(
            drawn.source, collector,
            "the console rehearses on the sample the module publishes, so a screen is never \
             drawn against a reading nobody checked"
        );
        let held = Golden::snapshot(collector)
            .held()
            .unwrap_or_else(|| panic!("{collector} has no shape on disk"));
        assert_eq!(Shape::of(&drawn).written(), held);
    }
}

#[test]
fn a_fixture_row_that_drifts_from_the_wire_is_caught_rather_than_drawn() {
    let sent = Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
        .with("timer|x", json!({"on_calendar": ["daily"]}));
    let drawn = Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
        .with("timer|x", json!({"on_calendar": "daily"}));

    assert_ne!(
        Shape::of(&sent),
        Shape::of(&drawn),
        "a list on the wire drawn as a string is the failure this comparison exists to catch"
    );
}

fn answers(name: &str, items: &std::collections::BTreeMap<String, Value>) {
    let shape = Shape::of_items(name, items);

    if let Err(complaint) = Golden::protocol(name).check("the console fixture", &shape.written()) {
        panic!("{complaint}");
    }
}

fn settled(name: &str) -> Settled {
    let sample = Golden::settled(name)
        .held()
        .unwrap_or_else(|| panic!("the console has no settled values of {name} to rehearse on"));

    Settled::read(&sample).expect("the sample parses")
}

#[test]
fn every_value_the_agent_has_one_right_answer_for_is_the_one_the_console_rehearses_on() {
    let missed = settled("status").missed(&super::answers::statuses());

    assert!(
        missed.is_empty(),
        "the console fixture answers for the agent where the agent has one right answer:\n{}\nThe \
         sample is {}; the agent writes it and the side to change is this one.",
        missed.join("\n"),
        Golden::settled("status").path().display()
    );
}

fn collector_rows() -> Vec<CollectorStatus> {
    let mut rows = super::agent::agent().collectors;
    rows.extend(super::answers::watching().collectors);
    if let Some(status) = super::view::view_with_launches().status {
        rows.extend(status.agent.collectors);
    }
    rows
}

#[test]
fn a_period_the_console_draws_is_the_one_the_collector_declares() {
    let mut shown: BTreeMap<String, Value> = BTreeMap::new();

    for row in collector_rows() {
        let Some(every_seconds) = row.every_seconds else {
            continue;
        };
        let period = json!({"every_seconds": every_seconds});
        if let Some(already) = shown.get(&row.name) {
            assert_eq!(
                already, &period,
                "the console fixture draws {} on two periods at once",
                row.name
            );
        }
        shown.insert(row.name, period);
    }

    let missed = settled("collectors").missed(&shown);
    assert!(
        missed.is_empty(),
        "the console rehearses on a period of its own:\n{}\nThe sample is {}, written from the \
         collectors this build ships; the side to change is this one.",
        missed.join("\n"),
        Golden::settled("collectors").path().display()
    );
}

#[test]
fn the_list_the_console_holds_is_the_size_the_agent_says_it_keeps() {
    let drawn = json!({"findings": {"capacity": super::view::view().found.capacity}});
    let missed = settled("status").missed_in("agent|watching", &drawn);

    assert!(
        missed.is_empty(),
        "the console holds a number of findings of its own:\n{}",
        missed.join("\n")
    );
}

#[test]
fn an_answer_the_console_was_never_shown_is_an_answer_it_never_drew() {
    answers("status", &super::answers::statuses());
    answers("refusal", &super::answers::refusals());
    answers("store", &super::answers::stores());
    answers("buffer", &super::answers::buffers());
}
