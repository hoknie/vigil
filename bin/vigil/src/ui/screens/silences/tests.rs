use std::path::PathBuf;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_config::{Source, Suppression};

use super::standing::Standing;
use super::{render, shown};
use crate::ui::helpers::words::text;
use crate::ui::{Silences, fixture};

fn silencing(key: &str, reason: &str) -> Suppression {
    Suppression {
        finding_key: Some(key.into()),
        reason: reason.into(),
        ..Suppression::default()
    }
}

fn held() -> Silences {
    Silences::read(Ok(vec![
        Source {
            path: PathBuf::from("/etc/vigil/vigil.yaml"),
            suppressions: Vec::new(),
        },
        Source {
            path: PathBuf::from("/etc/vigil/suppressions/10-deploy.yaml"),
            suppressions: vec![Suppression {
                finding_key_prefix: Some("port.listen|tcp|10.0.0.5:".into()),
                kind: Some("port.listen.new".into()),
                until: Some("2026-12-31T00:00:00.000Z".into()),
                reason: "the staging network".into(),
                ..Suppression::default()
            }],
        },
        Source {
            path: PathBuf::from("/etc/vigil/suppressions/console.yaml"),
            suppressions: vec![silencing(
                "user|group|docker",
                "the deploy user belongs there",
            )],
        },
    ]))
}

fn drawn(silences: &Silences, running: Option<&[String]>, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 12));
    render(silences, running, fixture::look(), buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn every_entry_is_drawn_with_what_it_covers_why_and_the_file_it_lives_in() {
    let silences = held();
    let running: Vec<String> = silences
        .written()
        .iter()
        .map(|row| row.suppression.describe())
        .collect();

    let page = drawn(&silences, Some(&running), 200);

    for said in [
        "port.listen|tcp|10.0.0.5:*",
        "port.listen.new",
        "2026-12-31 00:00",
        "the staging network",
        "10-deploy.yaml",
        "user|group|docker",
        "every kind",
        "console.yaml",
        "in force",
    ] {
        assert!(page.contains(said), "{said} is missing: {page}");
    }
}

#[test]
fn an_entry_the_agent_has_not_read_yet_says_so_rather_than_in_force() {
    let silences = held();
    let running = vec![silences.written()[0].suppression.describe()];

    let rows = shown(&silences, Some(&running));

    assert_eq!(rows[0].standing, Standing::InForce);
    assert_eq!(
        rows[1].standing,
        Standing::NotReadYet,
        "a line in a file is not yet a silence, and the list must not say it is"
    );
}

#[test]
fn an_entry_taken_out_of_its_file_that_the_agent_still_holds_stays_on_the_list_until_it_lets_go() {
    let silences = held();
    let mut running: Vec<String> = silences
        .written()
        .iter()
        .map(|row| row.suppression.describe())
        .collect();
    running.push("user|group|sudo — taken out a minute ago".into());

    let rows = shown(&silences, Some(&running));

    let gone = rows.last().expect("a row");
    assert_eq!(gone.standing, Standing::StillHeld);
    assert_eq!(
        gone.written, None,
        "there is nothing left in a file to take out"
    );
    assert!(
        drawn(&silences, Some(&running), 200).contains("still held"),
        "the agent goes on silencing it, and a list without it would say the host is louder \
         than it is"
    );
}

#[test]
fn with_no_answer_from_the_agent_nothing_is_called_in_force() {
    let rows = shown(&held(), None);

    assert!(rows.iter().all(|row| row.standing == Standing::Unasked));
}

#[test]
fn an_unreadable_file_is_said_and_not_drawn_as_a_host_that_silences_nothing() {
    let silences = Silences::read(Err("/etc/vigil/vigil.yaml: Permission denied".into()));

    let page = drawn(&silences, Some(&[]), 100);

    assert!(page.contains("could not be read"), "{page}");
    assert!(page.contains("Permission denied"), "{page}");
    assert!(!page.contains("Nothing is silenced"), "{page}");
}

#[test]
fn a_host_that_silences_nothing_says_so_and_says_how_to_start() {
    let page = drawn(&Silences::read(Ok(Vec::new())), Some(&[]), 100);

    assert!(page.contains("Nothing is silenced"), "{page}");
    assert!(page.contains("Press d on a finding"), "{page}");
}

#[test]
fn the_foot_of_the_list_names_the_file_the_selected_entry_is_taken_out_of() {
    let silences = held();

    let page = drawn(&silences, Some(&[]), 200);

    assert!(
        page.contains("u reports it again, taken out of /etc/vigil/suppressions/10-deploy.yaml"),
        "{page}"
    );
    assert!(page.contains("2 not read yet"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    let silences = held();
    for width in [60u16, 80, 120, 200] {
        for line in drawn(&silences, Some(&[]), width).lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
