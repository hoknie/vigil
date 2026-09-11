use serde_json::{Value, json};
use vigil_model::{Golden, Shape, Snapshot};

fn rehearses(who: &str, fixture: &Snapshot) {
    let shape = Shape::of(fixture);
    if let Err(complaint) = Golden::snapshot(&fixture.source).check(who, &shape.written()) {
        panic!("{complaint}");
    }
}

fn fixtures() -> Vec<(&'static str, Snapshot)> {
    vec![
        ("the ports fixture", super::sockets::snapshot()),
        ("the users fixture", super::accounts::accounts()),
        ("the processes fixture", super::programs::processes()),
        ("the persistence fixture", super::startup::persistence()),
        ("the launches fixture", super::launches::launches()),
    ]
}

#[test]
fn the_console_rehearses_on_the_shape_the_agent_actually_sends() {
    for (who, fixture) in fixtures() {
        rehearses(who, &fixture);
    }
}

#[test]
fn every_class_the_agent_can_send_has_a_row_on_some_screen() {
    for (who, fixture) in fixtures() {
        let sample = Golden::snapshot(&fixture.source)
            .held()
            .unwrap_or_else(|| panic!("{who} has no sample to rehearse on"));
        let published: Shape = serde_json::from_str(&sample).expect("the sample parses");
        let drawn = Shape::of(&fixture);

        let missing: Vec<&String> = published
            .classes
            .keys()
            .filter(|class| !drawn.classes.contains_key(*class))
            .collect();
        assert!(
            missing.is_empty(),
            "{who} carries no {missing:?} row and the agent sends one: a class nobody drew is a \
             screen that answers 'there is nothing here'"
        );
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

#[test]
fn an_answer_the_console_was_never_shown_is_an_answer_it_never_drew() {
    answers("status", &super::answers::statuses());
    answers("refusal", &super::answers::refusals());
    answers("store", &super::answers::stores());
}
