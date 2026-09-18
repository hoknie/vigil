use std::path::{Path, PathBuf};

use vigil_model::Finding;

use crate::Flow;
use crate::conformance::finding;
use crate::stores::files::{Limits, Outgoing};

fn temporary_path(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "vigil-outgoing-{}-{name}-{}.ndjson",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos()
                + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
            .unwrap_or(0)
    ))
}

fn small() -> Limits {
    Limits {
        findings: 10,
        journal_bytes: 64 * 1024,
    }
}

fn opened(path: &Path, limits: Limits) -> Outgoing {
    Outgoing::open("ndjson", path.to_path_buf(), limits).expect("opens")
}

fn findings(count: usize) -> Vec<Finding> {
    (0..count)
        .map(|index| finding(&format!("key-{index:04}"), "2026-09-09T10:00:00.000Z"))
        .collect()
}

fn keys(buffer: &Outgoing) -> Vec<String> {
    buffer
        .waiting()
        .into_iter()
        .map(|finding| finding.finding_key)
        .collect()
}

#[test]
fn what_no_receiver_took_is_still_waiting_after_a_restart() {
    let path = temporary_path("restart");
    {
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(3)).expect("keeps");
    }

    let buffer = opened(&path, small());

    assert_eq!(keys(&buffer), vec!["key-0000", "key-0001", "key-0002"]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_finding_the_receiver_took_does_not_come_back_a_second_time() {
    let path = temporary_path("taken");
    {
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(3)).expect("keeps");
        buffer.delivered(3).expect("hands over");
    }

    let buffer = opened(&path, small());

    assert!(
        buffer.is_empty(),
        "a buffer that replays what was accepted turns one finding into a finding per restart"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_delivery_that_stopped_half_way_leaves_the_rest_in_the_order_it_had() {
    let path = temporary_path("half");
    let mut buffer = opened(&path, small());
    buffer.keep(&findings(4)).expect("keeps");

    buffer.delivered(2).expect("two of four were taken");

    assert_eq!(keys(&buffer), vec!["key-0002", "key-0003"]);
    let reopened = opened(&path, small());
    assert_eq!(keys(&reopened), vec!["key-0002", "key-0003"], "on disk too");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_ceiling_displaces_the_oldest_and_says_so_rather_than_refusing_the_newest() {
    let path = temporary_path("ceiling");
    let mut buffer = opened(&path, small());

    buffer.keep(&findings(10)).expect("keeps");
    let flow = buffer.keep(&findings(4)[..1]).expect("keeps the eleventh");

    assert!(
        matches!(flow, Flow::Dropping { dropped, .. } if dropped > 0),
        "a buffer that silently loses the oldest makes lost data look like no data: {flow:?}"
    );
    assert!(buffer.waiting().len() <= small().findings);
    assert_eq!(
        keys(&buffer).last().map(String::as_str),
        Some("key-0000"),
        "what was kept is the newest, and the newest here is the one just written"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_buffer_that_emptied_after_it_had_lost_findings_says_so_once() {
    let path = temporary_path("drained");
    let mut buffer = opened(&path, small());
    buffer.keep(&findings(12)).expect("keeps past the ceiling");

    let drained = buffer.delivered(usize::MAX).expect("all of it was taken");
    buffer.keep(&findings(1)).expect("keeps one more");
    let after = buffer.delivered(1).expect("taken");

    assert!(
        matches!(drained, Flow::Drained { dropped } if dropped > 0),
        "{drained:?}"
    );
    assert_eq!(
        after,
        Flow::Steady,
        "the finding about a dropping buffer closes once, not on every delivery after it"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_buffer_still_holding_findings_has_not_drained_however_many_went_out() {
    let path = temporary_path("partial-drain");
    let mut buffer = opened(&path, small());
    buffer.keep(&findings(12)).expect("keeps past the ceiling");

    let flow = buffer.delivered(1).expect("one was taken");

    assert_eq!(flow, Flow::Steady, "{flow:?}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_buffer_over_its_byte_ceiling_is_compacted_rather_than_grown() {
    let path = temporary_path("bytes");
    let by_bytes = Limits {
        findings: 10_000,
        journal_bytes: 2_048,
    };
    let mut buffer = opened(&path, by_bytes);

    let flow = buffer.keep(&findings(40)).expect("keeps");

    assert!(matches!(flow, Flow::Dropping { .. }), "{flow:?}");
    assert!(
        buffer.held().bytes.held <= by_bytes.journal_bytes,
        "twelve enormous findings fill a buffer as surely as ten thousand small ones"
    );
    assert!(!buffer.is_empty(), "and the newest ones are still there");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_torn_last_line_costs_that_finding_and_not_the_whole_buffer() {
    use std::io::Write;

    let path = temporary_path("torn");
    {
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(2)).expect("keeps");
    }
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("reopens");
    file.write_all(b"{\"event_id\":\"half-writ").expect("tears");
    drop(file);

    let buffer = opened(&path, small());

    assert_eq!(keys(&buffer), vec!["key-0000", "key-0001"]);
    assert_eq!(
        buffer.damaged(),
        1,
        "the line nobody can read is counted and named, not thrown away in silence"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_file_left_over_a_ceiling_the_last_build_allowed_is_brought_under_it_on_open() {
    let path = temporary_path("shrunk");
    {
        let mut buffer = opened(
            &path,
            Limits {
                findings: 1_000,
                journal_bytes: 64 * 1024,
            },
        );
        buffer.keep(&findings(40)).expect("keeps");
    }

    let buffer = opened(&path, small());

    assert!(
        buffer.waiting().len() <= small().findings,
        "a ceiling that only applies to what this run wrote is not a ceiling"
    );
    assert_eq!(
        std::fs::read_to_string(&path)
            .expect("readable")
            .lines()
            .count(),
        buffer.waiting().len(),
        "and the file is what is held, not what was held"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn an_empty_buffer_names_no_oldest_finding_and_holds_no_bytes() {
    let path = temporary_path("empty");
    let buffer = opened(&path, small());

    let held = buffer.held();

    assert!(held.is_empty());
    assert_eq!(held.oldest_at, None);
    assert_eq!(held.bytes.held, 0);
    assert_eq!(buffer.name(), "ndjson");
    assert_eq!(
        buffer.path(),
        path,
        "and it names the file an operator reads"
    );
    let _ = std::fs::remove_file(&path);
}
