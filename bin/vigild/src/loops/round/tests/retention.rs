use vigil_collect::Health;
use vigil_model::Finding;
use vigil_store::Store;

use super::harness::{snapshot, watching};
use crate::helpers::rfc3339;

fn aged(port: u16, observed_at: &str) -> Finding {
    let mut finding = crate::socket::fixture::finding(&port.to_string());
    finding.finding_key = format!("port.listen|tcp|0.0.0.0:{port}");
    finding.observed_at = observed_at.to_string();
    finding.first_seen_at = observed_at.to_string();
    finding
}

fn dropped_past_the_window(round: &crate::loops::Round) -> u64 {
    round.shared.with(|state| {
        state
            .agent()
            .store
            .map(|store| store.dropped.past_the_window)
            .unwrap_or_default()
    })
}

#[test]
fn a_daemon_running_longer_than_the_window_forgets_what_aged_out_without_a_restart() {
    let it = watching("retention-aged", Health::Ok, vec![snapshot(&[443])]);
    it.round
        .store
        .record(&aged(4444, &rfc3339::days_ago(91)))
        .expect("records");
    it.round
        .store
        .record(&aged(8080, &rfc3339::days_ago(1)))
        .expect("records");

    it.round.take_retention();

    let journal = it.round.store.journal_path().expect("a journal on disk");
    let written = std::fs::read_to_string(journal).expect("readable");
    assert!(
        !written.contains("0.0.0.0:4444") && written.contains("0.0.0.0:8080"),
        "what aged out is gone from the file and what did not is still there: {written}"
    );
    assert_eq!(
        dropped_past_the_window(&it.round),
        1,
        "the console counts what the window took, the same as at start-up"
    );
}

#[test]
fn an_hour_in_which_nothing_aged_out_leaves_the_journal_as_it_was() {
    let it = watching("retention-nothing", Health::Ok, vec![snapshot(&[443])]);
    it.round
        .store
        .record(&aged(8080, &rfc3339::days_ago(1)))
        .expect("records");
    let journal = it.round.store.journal_path().expect("a journal on disk");
    let before = std::fs::read_to_string(&journal).expect("readable");
    let touched = std::fs::metadata(&journal)
        .and_then(|metadata| metadata.modified())
        .expect("a time");

    it.round.take_retention();

    assert_eq!(std::fs::read_to_string(&journal).expect("readable"), before);
    assert_eq!(
        std::fs::metadata(&journal)
            .and_then(|metadata| metadata.modified())
            .expect("a time"),
        touched,
        "a journal that lost nothing is not rewritten every hour"
    );
    assert_eq!(dropped_past_the_window(&it.round), 0);
}
