use std::path::Path;

use vigil_store::{FileStore, Store, StoreError};

use crate::helpers::rfc3339;
use crate::socket::Shared;
use crate::types::Policy;

pub fn open(state_dir: &Path) -> Result<FileStore, StoreError> {
    FileStore::open(state_dir)
}

pub fn prune(store: &dyn Store, retention_days: u32) {
    match store.prune(&rfc3339::days_ago(retention_days)) {
        Ok(0) => {}
        Ok(dropped) => {
            eprintln!("  history: {dropped} finding(s) past the retention window dropped")
        }
        Err(error) => eprintln!("  history: could not apply retention: {error}"),
    }
}

pub fn recall(store: &FileStore, shared: &Shared, policy: &Policy) {
    let capacity = shared.with(|state| state.findings_capacity());

    let held_open = match store.kept() {
        Ok(kept) => kept.open,
        Err(error) => {
            eprintln!("  history: not counted ({error}), the console starts with this run alone");
            return;
        }
    };
    let history = match store.open_findings(capacity) {
        Ok(history) => history,
        Err(error) => {
            eprintln!("  history: not read ({error}), the console starts with this run alone");
            return;
        }
    };

    let history: Vec<_> = history
        .into_iter()
        .filter(|finding| !policy.covers(finding, &finding.observed_at))
        .collect();

    let recalled = history.len();
    shared.with(|state| state.recall_findings(history, held_open));

    match held_open {
        0 => say_there_was_nothing(store.journal_absent_on_open()),
        _ => say_what_came_back(recalled, held_open, capacity),
    }
}

fn say_there_was_nothing(journal_absent: bool) {
    match journal_absent {
        true => eprintln!("  history: no findings journal yet, this run opens one"),
        false => eprintln!(
            "  history: nothing open in the findings journal, the console starts with this run alone"
        ),
    }
}

fn say_what_came_back(recalled: usize, held_open: u64, capacity: usize) {
    if held_open > recalled as u64 {
        eprintln!(
            "  history: {recalled} of {held_open} open finding(s) are on the console, newest first; \
             the console holds {capacity} and the journal keeps the rest"
        );
        return;
    }
    eprintln!("  history: {recalled} open finding(s) are on the console, newest first");
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use vigil_model::{Finding, State as FindingState};

    use super::*;
    use crate::socket::fixture;

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigild-history-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    fn aged(title: &str, observed_at: &str) -> Finding {
        let mut finding = fixture::finding(title);
        finding.finding_key = format!("port.listen|tcp|0.0.0.0:{title}");
        finding.observed_at = observed_at.to_string();
        finding.first_seen_at = observed_at.to_string();
        finding
    }

    fn console() -> Shared {
        Shared::new(fixture::state())
    }

    fn recorded(store: &FileStore, title: &str, observed_at: &str) {
        store.record(&aged(title, observed_at)).expect("records");
    }

    fn titles(shared: &Shared) -> Vec<String> {
        shared.with(|state| {
            state
                .latest_findings(None)
                .iter()
                .map(|finding| finding.title.clone())
                .collect()
        })
    }

    #[test]
    fn a_restart_opens_the_console_on_the_findings_the_last_run_wrote_down() {
        let directory = temporary_directory("restart");
        {
            let store = open(&directory).expect("opens");
            recorded(&store, "older", "2026-09-09T10:00:00.000Z");
            recorded(&store, "newer", "2026-09-09T11:00:00.000Z");
        }
        let store = open(&directory).expect("reopens");
        let shared = console();

        recall(&store, &shared, &Policy::new(Vec::new()));

        assert_eq!(
            titles(&shared),
            vec!["newer".to_string(), "older".to_string()],
            "a restart that shows nothing is a restart that lost the history it kept"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn what_the_retention_window_just_forgot_does_not_come_back_through_the_recall() {
        let directory = temporary_directory("pruned");
        let store = open(&directory).expect("opens");
        recorded(&store, "ancient", "2020-01-01T10:00:00.000Z");
        recorded(&store, "recent", "2026-09-09T11:00:00.000Z");
        let shared = console();

        prune(&store, 30);
        recall(&store, &shared, &Policy::new(Vec::new()));

        assert_eq!(
            titles(&shared),
            vec!["recent".to_string()],
            "the recall runs after the retention window, never before it"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_finding_closed_before_the_restart_does_not_come_back_open() {
        let directory = temporary_directory("closed");
        {
            let store = open(&directory).expect("opens");
            recorded(&store, "closed", "2026-09-09T10:00:00.000Z");
            recorded(&store, "still-open", "2026-09-09T10:30:00.000Z");
            assert!(
                store
                    .resolve(
                        "port.listen|tcp|0.0.0.0:closed",
                        "port.listen.new",
                        "2026-09-09T11:00:00.000Z"
                    )
                    .expect("resolves")
            );
        }
        let store = open(&directory).expect("reopens");
        let shared = console();

        recall(&store, &shared, &Policy::new(Vec::new()));

        assert_eq!(titles(&shared), vec!["still-open".to_string()]);
        assert!(
            shared.with(|state| state
                .latest_findings(None)
                .iter()
                .all(|finding| finding.state == FindingState::Open)),
            "a finding the last run closed must not read as one still standing"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_finding_the_operator_silenced_stays_silent_across_the_restart() {
        let directory = temporary_directory("silenced");
        {
            let store = open(&directory).expect("opens");
            recorded(&store, "silenced", "2026-09-09T10:00:00.000Z");
            recorded(&store, "loud", "2026-09-09T10:30:00.000Z");
        }
        let store = open(&directory).expect("reopens");
        let shared = console();
        let policy = Policy::new(vec![crate::Suppression {
            finding_key_prefix: Some("port.listen|tcp|0.0.0.0:silenced".into()),
            reason: "expected on this host".into(),
            ..Default::default()
        }]);

        recall(&store, &shared, &policy);

        assert_eq!(
            titles(&shared),
            vec!["loud".to_string()],
            "what the configuration silences is silent whichever run raised it"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_line_nobody_can_read_costs_that_line_and_not_the_rest_of_the_history() {
        let directory = temporary_directory("torn");
        {
            let store = open(&directory).expect("opens");
            recorded(&store, "intact", "2026-09-09T10:00:00.000Z");
        }
        let mut journal = fs::OpenOptions::new()
            .append(true)
            .open(directory.join("findings").join("journal.ndjson"))
            .expect("reopens the file");
        journal
            .write_all(b"{\"event_id\":\"half-writ")
            .expect("tears it");
        drop(journal);

        let store = open(&directory).expect("still opens");
        let shared = console();

        recall(&store, &shared, &Policy::new(Vec::new()));

        assert_eq!(titles(&shared), vec!["intact".to_string()]);
        assert_eq!(
            store.damaged_on_open(),
            1,
            "the line that could not be read is counted, not hidden"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_state_directory_with_no_journal_in_it_is_not_a_journal_that_holds_nothing() {
        let directory = temporary_directory("fresh");
        let store = open(&directory).expect("opens");
        let shared = console();

        recall(&store, &shared, &Policy::new(Vec::new()));

        assert!(titles(&shared).is_empty());
        assert!(
            store.journal_absent_on_open(),
            "a journal that is not there yet is not a journal that holds nothing"
        );
        assert_eq!(
            shared.with(|state| state.findings_dropped()),
            0,
            "an empty history hides nothing from the reader"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_journal_longer_than_the_console_holds_says_how_much_stays_behind() {
        let directory = temporary_directory("longer");
        let store = open(&directory).expect("opens");
        let shared = console();
        let capacity = shared.with(|state| state.findings_capacity());
        for index in 0..capacity + 7 {
            recorded(
                &store,
                &format!("key-{index:05}"),
                &format!("2026-09-09T10:00:00.{index:03}Z"),
            );
        }

        recall(&store, &shared, &Policy::new(Vec::new()));

        assert_eq!(titles(&shared).len(), capacity);
        assert_eq!(
            shared.with(|state| state.findings_dropped()),
            7,
            "what the journal holds beyond the screen has to be a number on the screen"
        );
        let _ = fs::remove_dir_all(&directory);
    }
}
