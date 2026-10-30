use serde_json::{Value, json};
use vigil_view::Piece;

use crate::views::detail::launch;

fn row(extra: Value) -> Value {
    let mut row = json!({
        "exe": "/usr/bin/nc",
        "user": "alice",
        "auid": 1000,
        "first_seen": "2026-09-09T12:00:03.412Z",
        "runs": 2,
    });
    for (key, value) in extra.as_object().expect("fields") {
        row[key] = value.clone();
    }
    row
}

fn field(pieces: &[Piece], named: &str) -> Option<String> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Field { name, value } if name == named => Some(value.clone()),
        _ => None,
    })
}

fn everything_said(pieces: &[Piece]) -> String {
    format!("{pieces:?}")
}

#[test]
fn the_card_names_the_audit_record_of_the_last_run_as_the_kernel_wrote_it() {
    let card = launch(&row(json!({
        "audit_id": "1757419203.412:3421",
        "last_audit_id": "1757419207.000:3425",
    })));

    assert_eq!(
        field(&card, "audit id").as_deref(),
        Some("1757419207.000:3425"),
        "the record last run names is the one the audit id names, so the two fields speak of \
         one line of the audit log"
    );

    let once = launch(&row(json!({"audit_id": "1757419203.412:3421"})));
    assert_eq!(
        field(&once, "audit id").as_deref(),
        Some("1757419203.412:3421")
    );
}

#[test]
fn a_row_with_no_audit_id_has_no_audit_id_field_rather_than_a_question_mark() {
    let card = launch(&row(json!({})));

    assert_eq!(field(&card, "audit id"), None);
}

#[test]
fn a_path_the_agent_was_not_shown_is_not_called_missing_from_disk() {
    let card = launch(&row(json!({"exe_present": null, "exe_shown": false})));

    assert_eq!(
        field(&card, "the file").as_deref(),
        Some("could not be checked: the agent was not shown that path when this record was read")
    );
    assert!(
        !everything_said(&card).contains("was not on disk"),
        "not seeing a file is not seeing it gone"
    );
}

#[test]
fn a_row_written_before_the_agent_checked_files_is_not_called_missing_from_disk() {
    let card = launch(&row(json!({})));

    assert_eq!(
        field(&card, "the file").as_deref(),
        Some("could not be checked: the agent was not shown that path when this record was read")
    );
}

#[test]
fn a_file_that_was_gone_is_still_called_gone() {
    let gone = launch(&row(json!({"exe_present": false, "exe_shown": true})));
    let there = launch(&row(json!({"exe_present": true, "exe_shown": true})));

    assert_eq!(
        field(&gone, "the file").as_deref(),
        Some("was not on disk when this record was read")
    );
    assert_eq!(
        field(&there, "the file").as_deref(),
        Some("was on disk when this record was read")
    );
}
