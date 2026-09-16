use serde_json::json;
use vigil_view::{Pane, Piece, Room, RowKey, Section, Showing, Sorting};

use crate::fixture::launches;
use crate::views::WhatHasRunHere;

const NC: &str = "run|alice|/usr/bin/nc.openbsd";

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

fn sorted(pane: &dyn Pane, header: &str, descending: bool) -> Sorting {
    let at = pane
        .sorted_by()
        .iter()
        .position(|by| *by == header)
        .expect("offered");
    Sorting::of(at * 2 + 1 + usize::from(descending))
}

fn launches_in(pane: &dyn Pane, sorting: Sorting) -> Vec<String> {
    pane.rows(&launches(), &Showing::default().sorted(sorting))
        .into_iter()
        .map(|row| row.key)
        .filter(|key| key.starts_with("run|"))
        .collect()
}

fn fields(pieces: &[Piece]) -> Vec<(String, String)> {
    pieces
        .iter()
        .filter_map(|piece| match piece {
            Piece::Field { name, value } => Some((name.clone(), value.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn sorted_by_last_run_the_program_run_most_recently_comes_first_and_the_earliest_last() {
    let pane = pane();

    let newest_first = launches_in(pane.as_ref(), sorted(pane.as_ref(), "LAST RUN", true));
    let oldest_first = launches_in(pane.as_ref(), sorted(pane.as_ref(), "LAST RUN", false));

    assert_eq!(
        newest_first.first().map(String::as_str),
        Some(NC),
        "alice ran nc first and last in the sample, and a list ordered by when it was first \
         seen would put her first run at the bottom: {newest_first:?}"
    );
    assert_eq!(
        oldest_first.first().map(String::as_str),
        Some("run|root|/dev/shm/payload"),
        "{oldest_first:?}"
    );
}

#[test]
fn sorted_by_runs_from_the_fewest_the_program_run_twice_comes_after_every_program_run_once() {
    let pane = pane();

    let fewest_first = launches_in(pane.as_ref(), sorted(pane.as_ref(), "RUNS", false));

    assert_eq!(
        fewest_first.last().map(String::as_str),
        Some(NC),
        "{fewest_first:?}"
    );
}

#[test]
fn a_launch_says_the_day_and_the_time_it_was_last_run_even_at_eighty_columns() {
    let reading = launches();

    let cells = pane().cells(&reading, &RowKey::of(NC), Room::of(80));

    assert_eq!(
        cells[3].text, "2025-09-09 12:00:07",
        "the kernel stamped alice's second run of nc at 1757419207, and a time of day with no \
         day says nothing about a program last run a month ago: {cells:?}"
    );
}

#[test]
fn the_history_of_a_launch_lists_the_moments_of_its_runs_newest_first() {
    let said = pane().history(&launches(), &RowKey::of(NC));
    let runs = fields(&said);

    let one = runs.iter().find(|(name, _)| name == "1").expect("a first");
    let two = runs.iter().find(|(name, _)| name == "2").expect("a second");
    assert!(
        one.1.contains("2025-09-09 12:00:07.000 UTC") && one.1.contains("nc -z"),
        "a run says when it ran and what was run: {said:?}"
    );
    assert!(
        two.1.contains("nc -l -p 4444"),
        "the same person ran the same program with other arguments, and that is what a reader \
         opens a history for: {said:?}"
    );
    assert!(
        !one.1.contains("1757419207.000:3425") && !one.1.contains("ausearch"),
        "the audit id is that moment and a number, and the moment is already written out beside \
         it: what a reader wants there is the command line: {said:?}"
    );
    assert!(
        two.1.contains("2025-09-09 12:00:03.412 UTC"),
        "the run the row was first written from is the oldest one: {said:?}"
    );
    assert!(
        !runs.iter().any(|(name, _)| name == "3"),
        "alice ran it twice: {said:?}"
    );
}

#[test]
fn the_history_of_a_row_about_the_reading_says_that_nothing_ran_under_it() {
    let said = format!(
        "{:?}",
        pane().history(&launches(), &RowKey::of("launches|capped"))
    );

    assert!(said.contains("nothing ran under it"), "{said}");
}

#[test]
fn a_row_an_older_agent_wrote_says_that_no_moment_was_kept_rather_than_listing_nothing() {
    let mut reading = launches();
    if let Some(fields) = reading
        .items
        .get_mut(NC)
        .and_then(serde_json::Value::as_object_mut)
    {
        fields.remove("recent_runs");
    }

    let said = pane().history(&reading, &RowKey::of(NC));

    assert!(
        format!("{said:?}").contains("No moment of a run is kept"),
        "{said:?}"
    );
    assert!(
        fields(&said)
            .iter()
            .any(|(name, value)| name == "last run" && value.contains("12:00:07")),
        "the last run is still known from the id it was counted by: {said:?}"
    );
}

#[test]
fn a_history_shorter_than_the_count_says_where_the_older_runs_are() {
    let mut reading = launches();
    reading.items.get_mut(NC).expect("in the sample")["runs"] = json!(20);

    let said = format!("{:?}", pane().history(&reading, &RowKey::of(NC)));

    assert!(said.contains("last 8 runs"), "{said}");
    assert!(said.contains("The 18 run(s) before them"), "{said}");
    assert!(
        said.contains("nc -l -p 4444"),
        "the runs it does keep say what was run: {said}"
    );
}

#[test]
fn the_launches_offer_a_history_of_their_rows() {
    assert!(pane().offers().history);
}

#[test]
fn a_moment_this_build_cannot_read_is_listed_by_its_audit_id_rather_than_dropped() {
    let mut reading = launches();
    reading.items.get_mut(NC).expect("in the sample")["recent_runs"] =
        json!(["not an audit id", "1757419207.000:3425"]);

    let said = pane().history(&reading, &RowKey::of(NC));
    let runs = fields(&said);

    assert!(
        runs.iter()
            .any(|(name, value)| name == "2" && value.contains("not an audit id")),
        "a row whose ids a later kernel wrote differently still lists what it kept, or the \
         panel counts runs it will not show: {said:?}"
    );
}
