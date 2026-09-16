use serde_json::{Value, json};

use super::super::snapshot::RECENT_RUNS;
use crate::parsers::audit::Execution;

use super::harness::{fresh, kept_on, later, launch, snapshot_of, with_arguments};

const NC: &str = "run|alice|/usr/bin/nc";

fn recent(item: &Value) -> Vec<&str> {
    item["recent_runs"]
        .as_array()
        .map(|runs| {
            runs.iter()
                .filter_map(|run| run.as_str().or_else(|| run["id"].as_str()))
                .collect()
        })
        .unwrap_or_default()
}

fn commands(item: &Value) -> Vec<Option<&str>> {
    item["recent_runs"]
        .as_array()
        .map(|runs| runs.iter().map(|run| run["arguments"].as_str()).collect())
        .unwrap_or_default()
}

#[test]
fn a_row_keeps_the_moments_of_eight_runs_and_no_more_because_every_row_is_held_in_memory() {
    assert_eq!(
        RECENT_RUNS, 8,
        "eight runs with the command line of each cost a row 977 bytes of the snapshot written \
         to disk, measured over five thousand rows of eight runs: 19.5 MB at the ceiling of \
         twenty thousand rows, and a reading of them merges in 10 ms rather than 6; raising it \
         is a decision with a number measured before and after, not an edit"
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

#[test]
fn a_run_keeps_the_command_line_it_was_run_with_so_the_history_says_what_ran() {
    let once = with_arguments(&[launch(1000, "/usr/bin/nc", &["nc", "-l", "-p", "4444"])]);
    let twice = kept_on(
        &once.items,
        &[later(1000, "/usr/bin/nc", &["nc", "-z", "host", "80"], 7)],
    );

    assert_eq!(
        commands(&twice.items[NC]),
        vec![Some("nc -l -p 4444"), Some("nc -z host 80")],
        "a row is one person and one program, and two runs of it are not the same command: \
         what a reader asks a history for is what changed between them"
    );
}

#[test]
fn a_command_line_longer_than_what_is_kept_is_cut_rather_than_held_whole() {
    let long = "x".repeat(400);
    let once = with_arguments(&[launch(1000, "/usr/bin/nc", &["nc", &long])]);
    let kept = commands(&once.items[NC])[0].expect("a command line");

    assert!(
        kept.chars().count() <= 121 && kept.ends_with('\u{2026}'),
        "eight command lines are held in memory for every row of the ledger, and a row is one \
         of twenty thousand: a command line is kept to be read, not to be stored whole: {}",
        kept.chars().count()
    );
}
