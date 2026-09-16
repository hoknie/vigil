use serde_json::{Value, json};

use super::super::snapshot::RECENT_RUNS;
use crate::parsers::audit::Execution;

use super::harness::{fresh, later, launch, snapshot_of};

const NC: &str = "run|alice|/usr/bin/nc";

fn recent(item: &Value) -> Vec<&str> {
    item["recent_runs"]
        .as_array()
        .map(|ids| ids.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

#[test]
fn a_row_keeps_the_moments_of_eight_runs_and_no_more_because_every_row_is_held_in_memory() {
    assert_eq!(
        RECENT_RUNS, 8,
        "eight audit ids cost a row 434 bytes of heap and 207 bytes of the snapshot written to \
         disk, measured over the ceiling of twenty thousand rows: 8.7 MB and 4.1 MB of the \
         agent's sixty-four; raising it is a decision with a number measured before and after, \
         not an edit"
    );

    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);
    let runs: Vec<Execution> = (0..100)
        .map(|serial| later(1000, "/usr/bin/nc", &["nc"], serial))
        .collect();
    let hundred = snapshot_of(&once.items, &runs);

    let expected: Vec<String> = (92..100)
        .map(|serial| format!("1757419300.000:{}", 4000 + serial))
        .collect();
    assert_eq!(
        recent(&hundred.items[NC]),
        expected,
        "the oldest moments are let go of and the last eight stay, in the order they ran"
    );
    assert_eq!(hundred.items[NC]["runs"], json!(101));
}

#[test]
fn the_first_run_of_a_row_is_the_first_moment_in_its_history() {
    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);

    assert_eq!(recent(&once.items[NC]), vec!["1757419203.412:3421"]);
}

#[test]
fn records_read_a_second_time_add_no_moment_to_the_history() {
    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);
    let tail = [
        launch(1000, "/usr/bin/nc", &["nc"]),
        later(1000, "/usr/bin/nc", &["nc"], 1),
    ];

    let read = snapshot_of(&once.items, &tail);
    let read_again = snapshot_of(&read.items, &tail);

    assert_eq!(
        recent(&read_again.items[NC]),
        vec!["1757419203.412:3421", "1757419300.000:4001"],
        "a run counted once is written down once, or the history lists restarts of the agent"
    );
}

#[test]
fn a_row_an_older_agent_wrote_without_a_history_starts_one_from_the_run_it_counted_last() {
    let mut known = snapshot_of(
        &fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]).items,
        &[later(1000, "/usr/bin/nc", &["nc"], 1)],
    )
    .items;
    if let Some(fields) = known.get_mut(NC).and_then(Value::as_object_mut) {
        fields.remove("recent_runs");
    }

    let again = snapshot_of(&known, &[later(1000, "/usr/bin/nc", &["nc"], 2)]);

    assert_eq!(
        recent(&again.items[NC]),
        vec!["1757419300.000:4001", "1757419300.000:4002"],
        "the last run the older agent counted is known, and the runs before it are not"
    );
}
