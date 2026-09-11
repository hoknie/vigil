use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::{Golden, Shape};

use super::answers::every_state;
use super::{refusals, snapshot, statuses, stores};
use crate::socket::switched_off_reason;

#[test]
fn the_daemon_rehearses_on_the_shape_it_sends() {
    let sent = snapshot();
    let shape = Shape::of(&sent);

    if let Err(complaint) =
        Golden::snapshot(&sent.source).check("the daemon fixture", &shape.written())
    {
        panic!("{complaint}");
    }
}

fn publishes(name: &str, items: &BTreeMap<String, Value>) {
    let shape = Shape::of_items(name, items);

    if let Err(complaint) = Golden::protocol(name).write_or_check(&shape.written()) {
        panic!("{complaint}");
    }
}

#[test]
fn an_answer_the_console_was_never_shown_is_an_answer_it_never_drew() {
    publishes("status", &statuses());
    publishes("refusal", &refusals());
    publishes("store", &stores());
}

#[test]
fn a_collector_that_is_switched_off_names_no_next_run_and_no_period() {
    let answers = statuses();
    let off = answers
        .keys()
        .find(|key| key.starts_with("collector-off|"))
        .expect("a collector the configuration switched off");
    let off = &answers[off];

    assert!(off["every_seconds"].is_null(), "{off}");
    assert!(off["next_run_at"].is_null(), "{off}");
    assert!(
        off["reason"].is_string(),
        "off without a word is indistinguishable from broken: {off}"
    );
}

#[test]
fn a_refusal_the_daemon_can_send_carries_the_state_the_summary_names() {
    let refusals = refusals();

    for state in ["degraded", "unavailable"] {
        assert!(
            refusals.values().any(|refusal| refusal["state"] == state),
            "no sample refusal is {state}: a screen for it is tested against nothing"
        );
    }
}

fn without_the_gaps(text: &str) -> String {
    text.chars()
        .filter(|letter| !letter.is_whitespace() && *letter != '\\')
        .collect()
}

fn what_the_console_rehearses_on() -> String {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vigil/src/ui/fixture");
    let mut text = String::new();

    for entry in std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
    {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_some_and(|kind| kind == "rs") {
            text.push_str(&std::fs::read_to_string(&path).expect("a fixture file"));
        }
    }

    without_the_gaps(&text)
}

#[test]
fn the_console_does_not_quote_the_daemon_word_for_word() {
    let rehearsal = what_the_console_rehearses_on();
    let mut said = every_state().agent().limitations;
    said.push(switched_off_reason("launches"));

    for sentence in said {
        assert!(
            !rehearsal.contains(&without_the_gaps(&sentence)),
            "the console fixture carries this daemon sentence word for word: {sentence:?}. The \
             daemon is the only author of its own wording, and the console answers for any \
             wording fitting on its screen. A copy here goes stale the day the daemon rewrites \
             the line, and nothing says so."
        );
    }
}
