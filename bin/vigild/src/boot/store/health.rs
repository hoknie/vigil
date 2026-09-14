use vigil_model::{Finding, KnownKind};
use vigil_store::Store;

use crate::loops::Watch;

pub fn standing(store: &dyn Store, watches: &[Watch]) -> Vec<(&'static str, String)> {
    let mut complaints = Vec::new();

    for watch in watches {
        let key = format!("agent.collector|{}", watch.name());
        match store.open_finding(&key, KnownKind::AgentCollectorDegraded.as_str()) {
            Ok(None) => {}
            Ok(Some(open)) => complaints.push((watch.name(), what_it_was(&open))),
            Err(error) => eprintln!("  history: {key} not read ({error}), so it stays open"),
        }
    }

    for (name, why) in &complaints {
        eprintln!("  history: {name} — a finding from a previous run stands: {why}");
    }

    complaints
}

fn what_it_was(open: &Finding) -> String {
    let said = open
        .evidence
        .iter()
        .find(|evidence| evidence.kind == "note")
        .map(|evidence| evidence.value.clone())
        .unwrap_or_else(|| open.title.clone());

    format!("{said} — first seen {}", open.first_seen_at)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use vigil_collect::{CollectError, Collector, Health};
    use vigil_model::Snapshot;
    use vigil_rules::RuleSet;
    use vigil_store::FileStore;

    use crate::helpers::agent_finding;

    use super::*;

    struct Answering(&'static str, Health);

    impl Collector for Answering {
        fn name(&self) -> &'static str {
            self.0
        }

        fn available(&self) -> Health {
            self.1.clone()
        }

        fn collect(&self) -> Result<Snapshot, CollectError> {
            Ok(Snapshot::new(self.0, "2026-09-11T09:00:00.000Z"))
        }
    }

    fn watching_a(health: Health) -> Vec<Watch> {
        vec![Watch::new(
            Box::new(Answering("launches", health)),
            RuleSet::of(Vec::new()),
        )]
    }

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "vigild-boot-health-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    fn degraded_in_the_last_run(store: &FileStore, detail: &str) {
        store
            .record(&agent_finding::collector_degraded(
                "launches", detail, false,
            ))
            .expect("records");
    }

    #[test]
    fn a_finding_the_last_run_left_open_is_named_so_that_something_can_close_it() {
        let directory = temporary_directory("standing");
        let store = FileStore::open(&directory).expect("opens");
        degraded_in_the_last_run(&store, "auditd has brought nothing");

        let complaints = standing(&store, &watching_a(Health::Ok));

        assert_eq!(complaints.len(), 1, "{complaints:?}");
        assert_eq!(complaints[0].0, "launches");
        assert!(
            complaints[0].1.contains("auditd has brought nothing"),
            "what it was is read out of the journal rather than guessed: {:?}",
            complaints[0].1
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_collector_nothing_was_ever_said_about_carries_nothing_to_close() {
        let directory = temporary_directory("quiet");
        let store = FileStore::open(&directory).expect("opens");

        let complaints = standing(&store, &watching_a(Health::Ok));

        assert!(
            complaints.is_empty(),
            "a healthy collector on a fresh host would otherwise report recovering from nothing"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_finding_the_last_run_already_closed_does_not_stand() {
        let directory = temporary_directory("already");
        let store = FileStore::open(&directory).expect("opens");
        degraded_in_the_last_run(&store, "auditd has brought nothing");
        store
            .resolve(
                "agent.collector|launches",
                KnownKind::AgentCollectorDegraded.as_str(),
                "2026-09-11T09:00:00.000Z",
            )
            .expect("resolves");

        let complaints = standing(&store, &watching_a(Health::Ok));

        assert!(
            complaints.is_empty(),
            "a collector that came back before the restart came back once"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_collector_that_is_still_unwell_carries_its_complaint_into_this_run() {
        let directory = temporary_directory("still");
        let store = FileStore::open(&directory).expect("opens");
        degraded_in_the_last_run(&store, "auditd has brought nothing");

        let complaints = standing(
            &store,
            &watching_a(Health::Degraded("auditd has brought nothing".into())),
        );

        assert_eq!(
            complaints.len(),
            1,
            "what stands stands whatever the collector says today: closing it is a decision \
             for the reading, not for the greeting"
        );
        let _ = fs::remove_dir_all(&directory);
    }
}
